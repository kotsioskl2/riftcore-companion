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
  const [donationPlatform, setDonationPlatform] = useState<'kofi' | 'revolut'>('kofi');
  const [selectedDonationAmount, setSelectedDonationAmount] = useState<number | 'custom'>(5);
  const [customDonationAmount, setCustomDonationAmount] = useState<string>('15');

  const KOFI_URL = 'https://ko-fi.com/X6I629C814';
  const REVOLUT_BASE = 'https://revolut.me/kkleovoulou';

  const getDonationUrl = () => {
    if (donationPlatform === 'kofi') {
      return KOFI_URL;
    }
    if (selectedDonationAmount === 'custom') {
      const val = parseFloat(customDonationAmount);
      return val > 0 ? `${REVOLUT_BASE}/${val}` : REVOLUT_BASE;
    }
    return `${REVOLUT_BASE}/${selectedDonationAmount}`;
  };

  const handleCopySupportLink = () => {
    navigator.clipboard.writeText(getDonationUrl());
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
          padding: '22px 24px',
          background: 'linear-gradient(135deg, rgba(0, 200, 255, 0.08) 0%, rgba(255, 42, 95, 0.05) 50%, rgba(6, 9, 15, 0.95) 100%)',
          border: '1px solid rgba(0, 200, 255, 0.35)',
          borderRadius: '12px',
          boxShadow: '0 0 28px rgba(0, 200, 255, 0.14)',
        }}
      >
        {/* Top Header */}
        <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginBottom: '14px', flexWrap: 'wrap', gap: '12px' }}>
          <div style={{ display: 'flex', alignItems: 'center', gap: '12px' }}>
            <div
              style={{
                width: '40px',
                height: '40px',
                borderRadius: '10px',
                background: 'rgba(0, 200, 255, 0.15)',
                border: '1px solid var(--border-blue)',
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'center',
                color: 'var(--blue-primary)',
                boxShadow: '0 0 15px rgba(0, 200, 255, 0.25)',
              }}
            >
              <Coffee size={22} />
            </div>
            <div>
              <div style={{ fontSize: '16px', fontWeight: 800, color: '#fff', display: 'flex', alignItems: 'center', gap: '8px' }}>
                <span>Fuel RiftCore Development</span>
                <Heart size={15} color="var(--red-primary)" fill="var(--red-primary)" />
              </div>
              <div style={{ fontSize: '11px', color: '#94a3b8', marginTop: '2px' }}>
                Zero ads. 100% free and open-source forever.
              </div>
            </div>
          </div>

          <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
            <button
              type="button"
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
              title="Copy link with current amount to clipboard"
            >
              {copiedLink ? <Check size={13} /> : <Copy size={13} />}
              <span>{copiedLink ? 'Link Copied!' : 'Copy Link'}</span>
            </button>
          </div>
        </div>

        {/* Creator Quote */}
        <p style={{ margin: '0 0 16px 0', fontSize: '12px', color: '#cbd5e1', fontStyle: 'italic', lineHeight: '1.6' }}>
          &ldquo;Building and hosting apps takes time and caffeine. If you want to help fuel the next project:&rdquo;
        </p>

        {/* Platform Selector Tabs */}
        <div style={{ display: 'flex', gap: '8px', marginBottom: '14px' }}>
          <button
            type="button"
            onClick={() => {
              setDonationPlatform('kofi');
              soundService.playTimerTick();
            }}
            style={{
              flex: 1,
              padding: '8px 12px',
              borderRadius: '8px',
              background: donationPlatform === 'kofi' ? 'linear-gradient(135deg, rgba(0, 200, 255, 0.2) 0%, rgba(0, 119, 255, 0.15) 100%)' : 'rgba(255, 255, 255, 0.03)',
              border: donationPlatform === 'kofi' ? '1px solid var(--blue-primary)' : '1px solid var(--border-subtle)',
              color: donationPlatform === 'kofi' ? '#fff' : '#94a3b8',
              fontSize: '12px',
              fontWeight: 700,
              cursor: 'pointer',
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
              gap: '6px',
              boxShadow: donationPlatform === 'kofi' ? '0 0 14px rgba(0, 200, 255, 0.25)' : 'none',
              transition: 'all 0.2s',
            }}
          >
            <Coffee size={14} color="var(--blue-primary)" />
            <span>Ko-fi (Apple Pay, G-Pay, Cards)</span>
            <span style={{ fontSize: '9px', background: 'rgba(34, 197, 94, 0.2)', color: '#22c55e', padding: '1px 5px', borderRadius: '4px', fontWeight: 800 }}>NO ACCOUNT NEEDED</span>
          </button>

          <button
            type="button"
            onClick={() => {
              setDonationPlatform('revolut');
              soundService.playTimerTick();
            }}
            style={{
              padding: '8px 14px',
              borderRadius: '8px',
              background: donationPlatform === 'revolut' ? 'linear-gradient(135deg, rgba(255, 42, 95, 0.2) 0%, rgba(255, 42, 95, 0.08) 100%)' : 'rgba(255, 255, 255, 0.03)',
              border: donationPlatform === 'revolut' ? '1px solid var(--red-primary)' : '1px solid var(--border-subtle)',
              color: donationPlatform === 'revolut' ? '#fff' : '#94a3b8',
              fontSize: '12px',
              fontWeight: 700,
              cursor: 'pointer',
              display: 'flex',
              alignItems: 'center',
              gap: '6px',
              boxShadow: donationPlatform === 'revolut' ? '0 0 14px rgba(255, 42, 95, 0.25)' : 'none',
              transition: 'all 0.2s',
            }}
          >
            <Zap size={14} color="var(--red-primary)" />
            <span>Revolut Me</span>
          </button>
        </div>

        {/* Interactive Donation Tier Boxes */}
        <div style={{ marginBottom: '16px' }}>
          <div style={{ fontSize: '11px', fontWeight: 800, textTransform: 'uppercase', letterSpacing: '0.6px', color: '#94a3b8', marginBottom: '10px', display: 'flex', alignItems: 'center', justifyContent: 'space-between' }}>
            <span>Choose Fuel Amount</span>
            <span style={{ color: 'var(--blue-primary)', fontSize: '10px', fontWeight: 700 }}>
              {donationPlatform === 'kofi' ? 'Pay via Apple Pay, Google Pay or Card' : 'Direct via Revolut'}
            </span>
          </div>

          <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(95px, 1fr))', gap: '8px' }}>
            {[
              { amount: 3, label: '€3', title: 'Espresso', icon: Coffee },
              { amount: 5, label: '€5', title: 'Energy Fuel', icon: Zap, popular: true },
              { amount: 10, label: '€10', title: 'Hextech Core', icon: Sparkles },
              { amount: 25, label: '€25', title: 'Overdrive', icon: Heart },
            ].map((box) => {
              const isSelected = selectedDonationAmount === box.amount;
              const IconComp = box.icon;
              return (
                <button
                  key={box.amount}
                  type="button"
                  onClick={() => {
                    setSelectedDonationAmount(box.amount);
                    soundService.playTimerTick();
                  }}
                  style={{
                    position: 'relative',
                    display: 'flex',
                    flexDirection: 'column',
                    alignItems: 'center',
                    justifyContent: 'center',
                    padding: '12px 8px',
                    borderRadius: '8px',
                    background: isSelected
                      ? 'linear-gradient(180deg, rgba(0, 200, 255, 0.22) 0%, rgba(0, 200, 255, 0.08) 100%)'
                      : 'rgba(255, 255, 255, 0.03)',
                    border: isSelected ? '1px solid var(--blue-primary)' : '1px solid var(--border-subtle)',
                    boxShadow: isSelected ? '0 0 16px rgba(0, 200, 255, 0.3)' : 'none',
                    cursor: 'pointer',
                    transition: 'all 0.2s cubic-bezier(0.4, 0, 0.2, 1)',
                  }}
                >
                  {box.popular && (
                    <span
                      style={{
                        position: 'absolute',
                        top: '-7px',
                        fontSize: '8px',
                        fontWeight: 900,
                        textTransform: 'uppercase',
                        padding: '1px 6px',
                        borderRadius: '10px',
                        background: 'linear-gradient(90deg, var(--blue-primary), #0077ff)',
                        color: '#000',
                        letterSpacing: '0.4px',
                      }}
                    >
                      POPULAR
                    </span>
                  )}
                  <IconComp size={16} color={isSelected ? 'var(--blue-primary)' : '#94a3b8'} style={{ marginBottom: '4px' }} />
                  <span style={{ fontSize: '16px', fontWeight: 900, color: isSelected ? '#fff' : '#cbd5e1' }}>
                    {box.label}
                  </span>
                  <span style={{ fontSize: '10px', fontWeight: 600, color: isSelected ? 'var(--blue-primary)' : '#64748b', marginTop: '2px' }}>
                    {box.title}
                  </span>
                </button>
              );
            })}

            {/* Custom Amount Box */}
            <button
              type="button"
              onClick={() => {
                setSelectedDonationAmount('custom');
                soundService.playTimerTick();
              }}
              style={{
                display: 'flex',
                flexDirection: 'column',
                alignItems: 'center',
                justifyContent: 'center',
                padding: '12px 8px',
                borderRadius: '8px',
                background: selectedDonationAmount === 'custom'
                  ? 'linear-gradient(180deg, rgba(255, 42, 95, 0.22) 0%, rgba(255, 42, 95, 0.08) 100%)'
                  : 'rgba(255, 255, 255, 0.03)',
                border: selectedDonationAmount === 'custom' ? '1px solid var(--red-primary)' : '1px solid var(--border-subtle)',
                boxShadow: selectedDonationAmount === 'custom' ? '0 0 16px rgba(255, 42, 95, 0.3)' : 'none',
                cursor: 'pointer',
                transition: 'all 0.2s ease',
              }}
            >
              <Heart size={16} color={selectedDonationAmount === 'custom' ? 'var(--red-primary)' : '#94a3b8'} style={{ marginBottom: '4px' }} />
              <span style={{ fontSize: '14px', fontWeight: 900, color: selectedDonationAmount === 'custom' ? '#fff' : '#cbd5e1' }}>
                Custom
              </span>
              <span style={{ fontSize: '10px', fontWeight: 600, color: selectedDonationAmount === 'custom' ? 'var(--red-primary)' : '#64748b', marginTop: '2px' }}>
                Choose
              </span>
            </button>
          </div>

          {/* Custom Input Field */}
          {selectedDonationAmount === 'custom' && (
            <div style={{ marginTop: '10px', display: 'flex', alignItems: 'center', gap: '8px' }}>
              <span style={{ fontSize: '14px', color: '#fff', fontWeight: 800 }}>€</span>
              <input
                type="number"
                min="1"
                placeholder="Enter custom EUR amount (e.g. 15, 50)"
                value={customDonationAmount}
                onChange={(e) => setCustomDonationAmount(e.target.value)}
                style={{
                  flex: 1,
                  padding: '8px 12px',
                  borderRadius: '6px',
                  background: 'rgba(0, 0, 0, 0.6)',
                  border: '1px solid var(--red-primary)',
                  color: '#fff',
                  fontSize: '13px',
                  fontWeight: 700,
                  outline: 'none',
                }}
              />
            </div>
          )}
        </div>

        {/* Action Buttons & Links */}
        <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', flexWrap: 'wrap', gap: '12px' }}>
          <div style={{ display: 'flex', alignItems: 'center', gap: '10px', flexWrap: 'wrap' }}>
            <a
              href={KOFI_URL}
              target="_blank"
              rel="noopener noreferrer"
              style={{
                display: 'inline-flex',
                alignItems: 'center',
                gap: '8px',
                padding: '10px 22px',
                borderRadius: '8px',
                background: 'linear-gradient(135deg, #13C3FF 0%, #0077ff 100%)',
                color: '#000',
                fontWeight: 900,
                fontSize: '13px',
                textDecoration: 'none',
                boxShadow: '0 0 20px rgba(0, 200, 255, 0.4)',
                transition: 'transform 0.15s, box-shadow 0.15s',
              }}
            >
              <Coffee size={16} fill="#000" />
              <span>
                Support {selectedDonationAmount === 'custom' ? `€${customDonationAmount || '0'}` : `€${selectedDonationAmount}`} on Ko-fi
              </span>
              <ExternalLink size={13} />
            </a>

            <a
              href={getDonationUrl()}
              target="_blank"
              rel="noopener noreferrer"
              style={{
                display: 'inline-flex',
                alignItems: 'center',
                gap: '6px',
                padding: '9px 16px',
                borderRadius: '8px',
                background: 'rgba(255, 255, 255, 0.06)',
                border: '1px solid var(--border-subtle)',
                color: '#cbd5e1',
                fontWeight: 700,
                fontSize: '12px',
                textDecoration: 'none',
              }}
              title="Donate with Revolut Me"
            >
              <Zap size={14} color="var(--red-primary)" />
              <span>Revolut</span>
            </a>
          </div>

          <div style={{ display: 'flex', alignItems: 'center', gap: '8px', fontSize: '11px', color: '#64748b' }}>
            <span>Ko-fi Page:</span>
            <code style={{ background: 'rgba(0,0,0,0.5)', padding: '3px 8px', borderRadius: '4px', color: 'var(--blue-primary)', fontFamily: 'var(--font-mono)' }}>
              ko-fi.com/X6I629C814
            </code>
          </div>
        </div>

        {/* Ko-fi QR Code & Mobile Fast-Scan Option */}
        <div
          style={{
            marginTop: '16px',
            padding: '12px 14px',
            borderRadius: '10px',
            background: 'rgba(0, 0, 0, 0.45)',
            border: '1px solid rgba(0, 200, 255, 0.2)',
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'space-between',
            flexWrap: 'wrap',
            gap: '14px',
          }}
        >
          <div style={{ display: 'flex', alignItems: 'center', gap: '14px' }}>
            <img
              src="./kofi_qr.png"
              alt="Ko-fi QR Code"
              style={{
                width: '68px',
                height: '68px',
                borderRadius: '8px',
                background: '#fff',
                padding: '4px',
                boxShadow: '0 0 16px rgba(0, 200, 255, 0.25)',
              }}
            />
            <div>
              <div style={{ fontSize: '13px', fontWeight: 800, color: '#fff', display: 'flex', alignItems: 'center', gap: '6px' }}>
                <span>Scan with Camera to Tip</span>
                <span style={{ fontSize: '9px', background: 'rgba(0, 200, 255, 0.15)', color: 'var(--blue-primary)', padding: '1px 6px', borderRadius: '4px', fontWeight: 800 }}>
                  APPLE PAY / G-PAY
                </span>
              </div>
              <div style={{ fontSize: '11px', color: '#94a3b8', marginTop: '3px', lineHeight: '1.4' }}>
                Point your phone camera at the QR code to tip in 3 seconds directly on mobile.
              </div>
            </div>
          </div>

          <a
            href={KOFI_URL}
            target="_blank"
            rel="noopener noreferrer"
            style={{
              padding: '6px 12px',
              borderRadius: '6px',
              background: 'rgba(0, 200, 255, 0.12)',
              border: '1px solid var(--border-blue)',
              color: 'var(--blue-primary)',
              fontSize: '11px',
              fontWeight: 700,
              textDecoration: 'none',
              display: 'flex',
              alignItems: 'center',
              gap: '6px',
            }}
          >
            <span>Open in Browser</span>
            <ExternalLink size={12} />
          </a>
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
