import React, { useState } from 'react';
import { Settings, Shield, Volume2, Zap, CheckCircle2, RotateCw, Monitor, Sparkles, Key, Globe, ExternalLink, Eye, EyeOff, AlertCircle } from 'lucide-react';
import { AppSettings } from '../types';
import { soundService } from '../services/soundService';
import { riotApiService, RIOT_REGIONS } from '../services/riotApiService';

interface SettingsModalProps {
  settings: AppSettings;
  onUpdateSetting: (key: keyof AppSettings, value: any) => void;
  isConnected: boolean;
  onReconnect: () => void;
}

export const SettingsModal: React.FC<SettingsModalProps> = ({
  settings,
  onUpdateSetting,
  isConnected,
  onReconnect,
}) => {
  const [apiKey, setApiKey] = useState(settings.riotApiKey || '');
  const [selectedRegion, setSelectedRegion] = useState(settings.riotRegion || 'eun1');
  const [showKey, setShowKey] = useState(false);
  const [isVerifying, setIsVerifying] = useState(false);
  const [verificationResult, setVerificationResult] = useState<{ valid: boolean; message: string } | null>(null);

  const handleVerifyAndSave = async () => {
    setIsVerifying(true);
    setVerificationResult(null);
    const result = await riotApiService.validateKey(apiKey, selectedRegion);
    setIsVerifying(false);
    setVerificationResult(result);
    if (result.valid) {
      soundService.playAutoAccept();
      onUpdateSetting('riotApiKey', apiKey.trim());
      onUpdateSetting('riotRegion', selectedRegion);
    }
  };
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

      {/* Official Riot Games Web API Integration Card */}
      <div className="glass-card" style={{ border: '1px solid var(--border-blue)' }}>
        <div className="card-header">
          <div className="card-title">
            <Key size={18} color="var(--blue-primary)" />
            <span>Official Riot Games Web API</span>
          </div>
          <span
            style={{
              fontSize: '11px',
              fontWeight: 800,
              color: settings.riotApiKey ? '#22c55e' : '#94a3b8',
              background: settings.riotApiKey ? 'rgba(34, 197, 94, 0.12)' : 'rgba(255, 255, 255, 0.05)',
              padding: '2px 8px',
              borderRadius: '4px',
              border: `1px solid ${settings.riotApiKey ? '#22c55e' : 'transparent'}`,
            }}
          >
            {settings.riotApiKey ? '● KEY REGISTERED' : 'OPTIONAL CLOUD TELEMETRY'}
          </span>
        </div>

        <div style={{ fontSize: '12px', color: '#94a3b8', marginBottom: '16px', lineHeight: '1.5' }}>
          Connect your personal or production API Key from{' '}
          <a
            href="https://developer.riotgames.com/"
            target="_blank"
            rel="noreferrer"
            style={{ color: 'var(--blue-primary)', textDecoration: 'underline', fontWeight: 600 }}
          >
            developer.riotgames.com <ExternalLink size={11} style={{ display: 'inline', verticalAlign: 'middle' }} />
          </a>{' '}
          to enable cross-region player lookups, cloud match histories, and multi-region ranked scouting.
        </div>

        <div style={{ display: 'flex', flexDirection: 'column', gap: '14px' }}>
          {/* Region Selector */}
          <div>
            <label style={{ fontSize: '11px', fontWeight: 700, color: '#cbd5e1', textTransform: 'uppercase', marginBottom: '6px', display: 'block' }}>
              Riot Regional Platform
            </label>
            <div style={{ position: 'relative' }}>
              <select
                value={selectedRegion}
                onChange={(e) => setSelectedRegion(e.target.value)}
                style={{
                  width: '100%',
                  padding: '8px 12px',
                  background: 'rgba(6, 9, 15, 0.85)',
                  border: '1px solid var(--border-subtle)',
                  borderRadius: '6px',
                  color: '#fff',
                  fontSize: '13px',
                  outline: 'none',
                  cursor: 'pointer',
                }}
              >
                {RIOT_REGIONS.map((r) => (
                  <option key={r.id} value={r.id} style={{ background: '#0a0e17', color: '#fff' }}>
                    {r.label}
                  </option>
                ))}
              </select>
            </div>
          </div>

          {/* API Key Input */}
          <div>
            <label style={{ fontSize: '11px', fontWeight: 700, color: '#cbd5e1', textTransform: 'uppercase', marginBottom: '6px', display: 'block' }}>
              Riot API Key (Starts with RGAPI-)
            </label>
            <div style={{ display: 'flex', gap: '8px' }}>
              <div style={{ position: 'relative', flex: 1 }}>
                <input
                  type={showKey ? 'text' : 'password'}
                  placeholder="RGAPI-xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx"
                  value={apiKey}
                  onChange={(e) => setApiKey(e.target.value)}
                  style={{
                    width: '100%',
                    padding: '8px 40px 8px 12px',
                    background: 'rgba(6, 9, 15, 0.85)',
                    border: '1px solid var(--border-subtle)',
                    borderRadius: '6px',
                    color: '#00c8ff',
                    fontFamily: 'var(--font-mono)',
                    fontSize: '12px',
                    outline: 'none',
                  }}
                />
                <button
                  type="button"
                  onClick={() => setShowKey(!showKey)}
                  style={{
                    position: 'absolute',
                    right: '8px',
                    top: '50%',
                    transform: 'translateY(-50%)',
                    background: 'none',
                    border: 'none',
                    color: '#94a3b8',
                    cursor: 'pointer',
                    display: 'flex',
                    alignItems: 'center',
                  }}
                  title={showKey ? 'Hide Key' : 'Show Key'}
                >
                  {showKey ? <EyeOff size={16} /> : <Eye size={16} />}
                </button>
              </div>

              <button
                className="btn-primary"
                onClick={handleVerifyAndSave}
                disabled={isVerifying || !apiKey.trim()}
                style={{
                  padding: '8px 18px',
                  fontSize: '12px',
                  fontWeight: 700,
                  display: 'flex',
                  alignItems: 'center',
                  gap: '6px',
                  opacity: isVerifying || !apiKey.trim() ? 0.6 : 1,
                }}
              >
                <CheckCircle2 size={14} />
                <span>{isVerifying ? 'Verifying...' : 'Verify & Connect'}</span>
              </button>
            </div>
          </div>

          {/* Verification Feedback Banner */}
          {verificationResult && (
            <div
              style={{
                padding: '10px 14px',
                borderRadius: '6px',
                fontSize: '12px',
                fontWeight: 600,
                display: 'flex',
                alignItems: 'center',
                gap: '8px',
                background: verificationResult.valid ? 'rgba(34, 197, 94, 0.12)' : 'rgba(255, 42, 95, 0.12)',
                border: `1px solid ${verificationResult.valid ? '#22c55e' : '#ff2a5f'}`,
                color: verificationResult.valid ? '#22c55e' : '#ff2a5f',
              }}
            >
              {verificationResult.valid ? <CheckCircle2 size={16} /> : <AlertCircle size={16} />}
              <span>{verificationResult.message}</span>
            </div>
          )}
        </div>
      </div>

      {/* Riot Games Compliance & Legal Jibber Jabber */}
      <div
        style={{
          padding: '14px 18px',
          background: 'rgba(255, 255, 255, 0.02)',
          border: '1px solid rgba(255, 255, 255, 0.06)',
          borderRadius: '8px',
          fontSize: '11px',
          color: '#64748b',
          lineHeight: '1.5',
        }}
      >
        <div style={{ fontWeight: 700, color: '#94a3b8', marginBottom: '4px' }}>
          ⚖️ Riot Games Developer Policy & Legal Jibber Jabber
        </div>
        RiftCore Companion isn't endorsed by Riot Games and doesn't reflect the views or opinions of Riot Games or anyone officially involved in producing or managing Riot Games properties. Riot Games, and all associated properties are trademarks or registered trademarks of Riot Games, Inc.
      </div>
    </div>
  );
};
