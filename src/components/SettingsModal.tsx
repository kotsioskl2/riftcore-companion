import React, { useState } from 'react';
import { Settings, Shield, Volume2, Zap, CheckCircle2, RotateCw, Monitor, Sparkles, Key, Globe, ExternalLink, Eye, EyeOff, AlertCircle, Minimize2, Keyboard, Coffee, Heart, Copy, Check } from 'lucide-react';
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
  const [hintReset, setHintReset] = useState(false);
  const [copiedLink, setCopiedLink] = useState(false);

  const handleCopySupportLink = () => {
    navigator.clipboard.writeText('https://revolut.me/kkleovoulou');
    setCopiedLink(true);
    soundService.playRuneImport();
    setTimeout(() => setCopiedLink(false), 2500);
  };

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

          {/* Auto Export In-Game Item Sets (Blitz Style) */}
          <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', paddingBottom: '14px', borderBottom: '1px solid var(--border-subtle)' }}>
            <div>
              <div style={{ fontSize: '14px', fontWeight: 600, color: '#fff', display: 'flex', alignItems: 'center', gap: '8px' }}>
                <span>Auto Export In-Game Item Sets</span>
                <span style={{ fontSize: '10px', background: 'rgba(0, 200, 255, 0.15)', color: 'var(--blue-primary)', padding: '2px 6px', borderRadius: '4px', fontWeight: 700 }}>
                  BLITZ STYLE
                </span>
              </div>
              <div style={{ fontSize: '12px', color: '#94a3b8', marginTop: '2px' }}>
                Automatically injects personalized item builds directly into your League in-game shop (Press 'P' in game).
              </div>
            </div>
            <div
              className={`toggle-switch ${settings.autoImportItemSets ? 'on' : ''}`}
              onClick={() => onUpdateSetting('autoImportItemSets', !settings.autoImportItemSets)}
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

      {/* Windows System Tray & Background Automation Card */}
      <div className="glass-card">
        <div className="card-header">
          <div className="card-title">
            <Minimize2 size={18} color="var(--blue-primary)" />
            <span>Windows System Tray & Background Operation</span>
          </div>
          <span
            style={{
              fontSize: '11px',
              fontWeight: 700,
              color: '#00c8ff',
              background: 'rgba(0, 200, 255, 0.1)',
              padding: '2px 8px',
              borderRadius: '4px',
              border: '1px solid rgba(0, 200, 255, 0.3)',
            }}
          >
            Tray Active
          </span>
        </div>

        <div style={{ display: 'flex', flexDirection: 'column', gap: '16px' }}>
          {/* Close to Tray Toggle */}
          <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', paddingBottom: '14px', borderBottom: '1px solid var(--border-subtle)' }}>
            <div>
              <div style={{ fontSize: '14px', fontWeight: 600, color: '#fff' }}>
                Minimize to System Tray on Close (X)
              </div>
              <div style={{ fontSize: '12px', color: '#94a3b8', marginTop: '2px' }}>
                Hides window into the Windows taskbar tray when clicking (X) so Auto-Accept and In-Game HUD stay active.
              </div>
            </div>
            <div
              className={`toggle-switch ${settings.closeToTray !== false ? 'on' : ''}`}
              onClick={() => onUpdateSetting('closeToTray', settings.closeToTray === false)}
            >
              <div className="toggle-knob" />
            </div>
          </div>

          {/* Reset Confirmation Dialog */}
          <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', paddingBottom: '14px', borderBottom: '1px solid var(--border-subtle)' }}>
            <div>
              <div style={{ fontSize: '14px', fontWeight: 600, color: '#fff' }}>
                Close Confirmation Dialog
              </div>
              <div style={{ fontSize: '12px', color: '#94a3b8', marginTop: '2px' }}>
                Reset your saved choice if you want RiftCore to ask before minimizing to tray when clicking (X).
              </div>
            </div>
            <button
              className="btn-secondary"
              style={{ fontSize: '11px', padding: '5px 12px' }}
              onClick={() => {
                localStorage.removeItem('riftcore_tray_hint_dismissed');
                setHintReset(true);
                setTimeout(() => setHintReset(false), 2500);
              }}
            >
              {hintReset ? (
                <>
                  <CheckCircle2 size={12} color="#22c55e" />
                  <span style={{ color: '#22c55e' }}>Choice Reset!</span>
                </>
              ) : (
                <>
                  <RotateCw size={12} />
                  <span>Reset Prompt</span>
                </>
              )}
            </button>
          </div>

          {/* Shortcuts Info */}
          <div
            style={{
              padding: '10px 14px',
              borderRadius: '6px',
              background: 'rgba(0, 200, 255, 0.05)',
              border: '1px solid rgba(0, 200, 255, 0.15)',
              display: 'flex',
              alignItems: 'center',
              gap: '12px',
              fontSize: '12px',
            }}
          >
            <Keyboard size={18} color="var(--blue-primary)" style={{ flexShrink: 0 }} />
            <div style={{ color: '#cbd5e1' }}>
              <strong>In-Game Hotkeys:</strong> Press <kbd style={{ padding: '2px 6px', background: 'rgba(255,255,255,0.1)', borderRadius: '4px', border: '1px solid rgba(255,255,255,0.2)' }}>Ctrl + Space</kbd> or <kbd style={{ padding: '2px 6px', background: 'rgba(255,255,255,0.1)', borderRadius: '4px', border: '1px solid rgba(255,255,255,0.2)' }}>Alt + ~</kbd> anytime during matches to toggle the Compact HUD overlay.
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

      {/* Fuel RiftCore Development & Creator Support */}
      <div
        className="glass-card"
        style={{
          padding: '20px 22px',
          background: 'linear-gradient(135deg, rgba(0, 200, 255, 0.06) 0%, rgba(255, 42, 95, 0.05) 50%, rgba(6, 9, 15, 0.9) 100%)',
          border: '1px solid rgba(0, 200, 255, 0.3)',
          borderRadius: '10px',
          boxShadow: '0 0 24px rgba(0, 200, 255, 0.12)',
        }}
      >
        <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginBottom: '12px', flexWrap: 'wrap', gap: '12px' }}>
          <div style={{ display: 'flex', alignItems: 'center', gap: '10px' }}>
            <div
              style={{
                width: '36px',
                height: '36px',
                borderRadius: '8px',
                background: 'rgba(0, 200, 255, 0.15)',
                border: '1px solid var(--border-blue)',
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'center',
                color: 'var(--blue-primary)',
              }}
            >
              <Coffee size={20} />
            </div>
            <div>
              <div style={{ fontSize: '15px', fontWeight: 700, color: '#fff', display: 'flex', alignItems: 'center', gap: '8px' }}>
                <span>Fuel RiftCore Development</span>
                <Heart size={14} color="var(--red-primary)" fill="var(--red-primary)" />
              </div>
              <div style={{ fontSize: '11px', color: '#94a3b8' }}>
                Zero ads. 100% free and open-source forever.
              </div>
            </div>
          </div>

          <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
            <button
              onClick={handleCopySupportLink}
              style={{
                padding: '7px 12px',
                borderRadius: '6px',
                background: copiedLink ? 'rgba(34, 197, 94, 0.15)' : 'rgba(255, 255, 255, 0.05)',
                border: copiedLink ? '1px solid #22c55e' : '1px solid var(--border-subtle)',
                color: copiedLink ? '#22c55e' : '#cbd5e1',
                fontSize: '11px',
                fontWeight: 600,
                cursor: 'pointer',
                display: 'flex',
                alignItems: 'center',
                gap: '6px',
                transition: 'all 0.2s',
              }}
              title="Copy Revolut support URL to clipboard"
            >
              {copiedLink ? <Check size={13} /> : <Copy size={13} />}
              <span>{copiedLink ? 'Link Copied!' : 'Copy Link'}</span>
            </button>

            <a
              href="https://revolut.me/kkleovoulou"
              target="_blank"
              rel="noopener noreferrer"
              style={{
                display: 'inline-flex',
                alignItems: 'center',
                gap: '6px',
                padding: '7px 14px',
                borderRadius: '6px',
                background: 'linear-gradient(135deg, #00c8ff 0%, #0077ff 100%)',
                color: '#000',
                fontWeight: 800,
                fontSize: '11px',
                textDecoration: 'none',
                boxShadow: '0 0 16px rgba(0, 200, 255, 0.35)',
              }}
            >
              <span>💻 revolut.me/kkleovoulou</span>
              <ExternalLink size={12} />
            </a>
          </div>
        </div>

        <p style={{ margin: '0 0 10px 0', fontSize: '12px', color: '#cbd5e1', fontStyle: 'italic', lineHeight: '1.5' }}>
          &ldquo;Building and hosting apps takes time and caffeine. If you want to help fuel the next project:&rdquo;
        </p>

        <div style={{ display: 'flex', alignItems: 'center', gap: '8px', fontSize: '11px', color: '#64748b' }}>
          <span>Direct Link:</span>
          <code style={{ background: 'rgba(0,0,0,0.5)', padding: '3px 8px', borderRadius: '4px', color: 'var(--blue-primary)', fontFamily: 'var(--font-mono)' }}>
            revolut.me/kkleovoulou
          </code>
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
