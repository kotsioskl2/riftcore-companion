import React, { useState } from 'react';
import {
  Settings,
  Shield,
  Volume2,
  Zap,
  CheckCircle2,
  RotateCw,
  Monitor,
  Sparkles,
  Key,
  Globe,
  ExternalLink,
  Eye,
  EyeOff,
  AlertCircle,
  Minimize2,
  Keyboard,
  Coffee,
  Heart,
  Copy,
  Check,
  Type,
  ArrowDownToLine,
  Loader2,
  Sliders,
  Palette,
  Activity,
  ChevronRight,
  Info
} from 'lucide-react';
import { AppSettings, AppUpdateInfo } from '../types';
import { soundService } from '../services/soundService';
import { riotApiService, RIOT_REGIONS } from '../services/riotApiService';
import { APP_VERSION } from '../version';

export type SettingsTab = 'automation' | 'hud' | 'appearance' | 'system' | 'updates';

interface SettingsModalProps {
  settings: AppSettings;
  onUpdateSetting: (key: keyof AppSettings, value: any) => void;
  isConnected: boolean;
  onReconnect: () => void;
  appUpdate?: AppUpdateInfo | null;
  onCheckForUpdate?: () => Promise<any>;
}

export const SettingsModal: React.FC<SettingsModalProps> = ({
  settings,
  onUpdateSetting,
  isConnected,
  onReconnect,
  appUpdate,
  onCheckForUpdate,
}) => {
  const [activeTab, setActiveTab] = useState<SettingsTab>('automation');
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
  const [isCheckingUpdate, setIsCheckingUpdate] = useState(false);
  const [updateCheckMsg, setUpdateCheckMsg] = useState<string | null>(null);
  const [isUpdatingFromSettings, setIsUpdatingFromSettings] = useState(false);

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

  const handleCheckUpdateClick = async () => {
    if (!onCheckForUpdate) return;
    setIsCheckingUpdate(true);
    setUpdateCheckMsg(null);
    try {
      const res = await onCheckForUpdate();
      if (res && res.hasUpdate) {
        setUpdateCheckMsg(`Update available: v${res.latestVersion}!`);
      } else {
        setUpdateCheckMsg(`You are on the latest version! (v${res?.currentVersion || APP_VERSION})`);
      }
    } catch {
      setUpdateCheckMsg('Could not reach GitHub Releases.');
    } finally {
      setIsCheckingUpdate(false);
    }
  };

  const handleApplyUpdateClick = async (url: string) => {
    if (!window.riftAPI?.downloadAppUpdate) return;
    setIsUpdatingFromSettings(true);
    try {
      await window.riftAPI.downloadAppUpdate(url);
    } catch (e: any) {
      setUpdateCheckMsg(e?.message || 'Update failed');
      setIsUpdatingFromSettings(false);
    }
  };

  const tabs: { id: SettingsTab; label: string; icon: React.FC<any>; badge?: string; badgeColor?: string }[] = [
    { id: 'automation', label: 'Automation', icon: Zap },
    { id: 'hud', label: 'In-Game HUD', icon: Monitor, badge: '8 Modules' },
    { id: 'appearance', label: 'Appearance', icon: Palette },
    { id: 'system', label: 'System & API', icon: Settings },
    {
      id: 'updates',
      label: 'Updates & About',
      icon: Sparkles,
      badge: appUpdate?.hasUpdate ? 'UPDATE' : undefined,
      badgeColor: '#10b981',
    },
  ];

  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: '16px', maxWidth: '820px', margin: '0 auto', width: '100%' }}>
      {/* Reorganized Modern Category Navigation Strip */}
      <div
        style={{
          display: 'flex',
          alignItems: 'center',
          gap: '6px',
          background: 'rgba(8, 13, 24, 0.72)',
          border: '1px solid rgba(255, 255, 255, 0.08)',
          borderRadius: '10px',
          padding: '6px',
          backdropFilter: 'blur(16px)',
          overflowX: 'auto',
          boxShadow: '0 4px 20px rgba(0, 0, 0, 0.25)',
        }}
      >
        {tabs.map((tab) => {
          const IconComp = tab.icon;
          const isActive = activeTab === tab.id;
          return (
            <button
              key={tab.id}
              onClick={() => {
                setActiveTab(tab.id);
                soundService.playTimerTick();
              }}
              style={{
                flex: 1,
                minWidth: '120px',
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'center',
                gap: '8px',
                padding: '9px 14px',
                borderRadius: '8px',
                border: isActive ? '1px solid var(--blue-primary)' : '1px solid transparent',
                background: isActive
                  ? 'linear-gradient(135deg, rgba(0, 210, 255, 0.16) 0%, rgba(0, 119, 255, 0.1) 100%)'
                  : 'transparent',
                color: isActive ? '#fff' : '#94a3b8',
                fontSize: '12px',
                fontWeight: isActive ? 800 : 600,
                cursor: 'pointer',
                transition: 'all 0.18s cubic-bezier(0.2, 0, 0, 1)',
                boxShadow: isActive ? '0 0 14px rgba(0, 210, 255, 0.22)' : 'none',
                whiteSpace: 'nowrap',
              }}
            >
              <IconComp size={15} color={isActive ? 'var(--blue-primary)' : '#64748b'} />
              <span>{tab.label}</span>
              {tab.badge && (
                <span
                  style={{
                    fontSize: '9px',
                    fontWeight: 900,
                    padding: '1px 5px',
                    borderRadius: '4px',
                    background: tab.badgeColor ? 'rgba(16, 185, 129, 0.2)' : 'rgba(0, 210, 255, 0.15)',
                    color: tab.badgeColor || 'var(--blue-primary)',
                    border: `1px solid ${tab.badgeColor ? 'rgba(16, 185, 129, 0.4)' : 'rgba(0, 210, 255, 0.3)'}`,
                    letterSpacing: '0.4px',
                  }}
                >
                  {tab.badge}
                </span>
              )}
            </button>
          );
        })}
      </div>

      {/* ====================================================================== */}
      {/* TAB 1: AUTOMATION & LEAGUE CLIENT                                      */}
      {/* ====================================================================== */}
      {activeTab === 'automation' && (
        <div style={{ display: 'flex', flexDirection: 'column', gap: '16px' }}>
          <div className="glass-card">
            <div className="card-header">
              <div className="card-title">
                <Zap size={17} color="var(--blue-primary)" />
                <span>Client Automation & Workflow</span>
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
                100% Vanguard Safe
              </span>
            </div>

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

              {/* Auto Export In-Game Item Sets */}
              <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', paddingBottom: '14px', borderBottom: '1px solid var(--border-subtle)' }}>
                <div>
                  <div style={{ fontSize: '14px', fontWeight: 600, color: '#fff' }}>
                    Auto Export In-Game Item Sets
                  </div>
                  <div style={{ fontSize: '12px', color: '#94a3b8', marginTop: '2px' }}>
                    Automatically syncs recommended item builds directly into your League in-game shop (Press 'P' in game).
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
              <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between' }}>
                <div>
                  <div style={{ fontSize: '14px', fontWeight: 600, color: '#fff' }}>
                    Hextech Audio Cues
                  </div>
                  <div style={{ fontSize: '12px', color: '#94a3b8', marginTop: '2px' }}>
                    Play subtle audio chimes on Match Found, Runes Applied, and Cooldown resets.
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
        </div>
      )}

      {/* ====================================================================== */}
      {/* TAB 2: IN-GAME HUD OVERLAYS                                            */}
      {/* ====================================================================== */}
      {activeTab === 'hud' && (
        <div style={{ display: 'flex', flexDirection: 'column', gap: '16px' }}>
          {/* Hotkey Guide Banner */}
          <div
            style={{
              padding: '12px 16px',
              borderRadius: '8px',
              background: 'linear-gradient(135deg, rgba(0, 210, 255, 0.08) 0%, rgba(14, 165, 233, 0.04) 100%)',
              border: '1px solid rgba(0, 210, 255, 0.2)',
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'space-between',
              gap: '12px',
              flexWrap: 'wrap',
            }}
          >
            <div style={{ display: 'flex', alignItems: 'center', gap: '10px' }}>
              <Keyboard size={18} color="var(--blue-primary)" />
              <span style={{ fontSize: '12.5px', color: '#e2e8f0' }}>
                Toggle Fullscreen HUD in-game: <kbd style={{ padding: '2px 7px', background: 'rgba(0,0,0,0.4)', borderRadius: '4px', border: '1px solid rgba(0,210,255,0.3)', color: '#00d2ff', fontWeight: 800 }}>Alt + G</kbd> or <kbd style={{ padding: '2px 7px', background: 'rgba(0,0,0,0.4)', borderRadius: '4px', border: '1px solid rgba(255,255,255,0.2)', color: '#fff', fontWeight: 800 }}>Ctrl + Space</kbd>
              </span>
            </div>
            <span style={{ fontSize: '11px', color: '#94a3b8' }}>Zero click delay · Vanguard compliant</span>
          </div>

          <div className="glass-card">
            <div className="card-header">
              <div className="card-title">
                <Monitor size={18} color="var(--blue-primary)" />
                <span>Tactical HUD Modules (Toggle On/Off)</span>
              </div>
              <span
                style={{
                  fontSize: '11px',
                  fontWeight: 700,
                  color: 'var(--blue-primary)',
                  background: 'rgba(0, 210, 255, 0.1)',
                  padding: '2px 8px',
                  borderRadius: '4px',
                  border: '1px solid rgba(0, 210, 255, 0.25)',
                }}
              >
                Live Overlay Chips
              </span>
            </div>

            <div style={{ display: 'flex', flexDirection: 'column', gap: '14px' }}>
              {/* 1. Team Gold Lead */}
              <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', paddingBottom: '12px', borderBottom: '1px solid var(--border-subtle)' }}>
                <div>
                  <div style={{ fontSize: '13.5px', fontWeight: 700, color: '#fff', display: 'flex', alignItems: 'center', gap: '6px' }}>
                    <span>🪙 Live Scoreboard & Team Gold Lead</span>
                  </div>
                  <div style={{ fontSize: '11.5px', color: '#94a3b8', marginTop: '2px' }}>
                    Displays real-time net team gold advantage and lane gold differential at the top center of your screen.
                  </div>
                </div>
                <div
                  className={`toggle-switch ${settings.hudGoldLead !== false ? 'on' : ''}`}
                  onClick={() => onUpdateSetting('hudGoldLead', settings.hudGoldLead === false ? true : false)}
                >
                  <div className="toggle-knob" />
                </div>
              </div>

              {/* 2. CS Benchmark */}
              <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', paddingBottom: '12px', borderBottom: '1px solid var(--border-subtle)' }}>
                <div>
                  <div style={{ fontSize: '13.5px', fontWeight: 700, color: '#fff', display: 'flex', alignItems: 'center', gap: '6px' }}>
                    <span>🌾 CS Benchmark & Wave Pace Rating</span>
                  </div>
                  <div style={{ fontSize: '11.5px', color: '#94a3b8', marginTop: '2px' }}>
                    Tracks CS/min against Challenger targets (8.4+ CS/min) with lane opponent creep score difference.
                  </div>
                </div>
                <div
                  className={`toggle-switch ${settings.hudCsBenchmark !== false ? 'on' : ''}`}
                  onClick={() => onUpdateSetting('hudCsBenchmark', settings.hudCsBenchmark === false ? true : false)}
                >
                  <div className="toggle-knob" />
                </div>
              </div>

              {/* 3. Objective Timers */}
              <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', paddingBottom: '12px', borderBottom: '1px solid var(--border-subtle)' }}>
                <div>
                  <div style={{ fontSize: '13.5px', fontWeight: 700, color: '#fff', display: 'flex', alignItems: 'center', gap: '6px' }}>
                    <span>🐉 Dragon, Baron & Camp Respawn Timers</span>
                  </div>
                  <div style={{ fontSize: '11.5px', color: '#94a3b8', marginTop: '2px' }}>
                    Live countdown timers for Elemental Dragon, Baron Nashor, Voidgrubs, and Red/Blue buff camps over your minimap.
                  </div>
                </div>
                <div
                  className={`toggle-switch ${settings.hudObjectiveTimers !== false ? 'on' : ''}`}
                  onClick={() => onUpdateSetting('hudObjectiveTimers', settings.hudObjectiveTimers === false ? true : false)}
                >
                  <div className="toggle-knob" />
                </div>
              </div>

              {/* 4. Enemy Summoner Spell Tracker */}
              <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', paddingBottom: '12px', borderBottom: '1px solid var(--border-subtle)' }}>
                <div>
                  <div style={{ fontSize: '13.5px', fontWeight: 700, color: '#fff', display: 'flex', alignItems: 'center', gap: '6px' }}>
                    <span>⚡ Enemy Summoner Spell Tracker</span>
                  </div>
                  <div style={{ fontSize: '11.5px', color: '#94a3b8', marginTop: '2px' }}>
                    Interactive click-to-track enemy Flash, Ghost, Teleport, and Ignite cooldowns with one-click chat sync.
                  </div>
                </div>
                <div
                  className={`toggle-switch ${settings.hudSpellTimers !== false ? 'on' : ''}`}
                  onClick={() => onUpdateSetting('hudSpellTimers', settings.hudSpellTimers === false ? true : false)}
                >
                  <div className="toggle-knob" />
                </div>
              </div>

              {/* 5. Next Skill Upgrade Pointer */}
              <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', paddingBottom: '12px', borderBottom: '1px solid var(--border-subtle)' }}>
                <div>
                  <div style={{ fontSize: '13.5px', fontWeight: 700, color: '#fff', display: 'flex', alignItems: 'center', gap: '6px' }}>
                    <span>🏹 Next Skill Upgrade Ability Pointer</span>
                  </div>
                  <div style={{ fontSize: '11.5px', color: '#94a3b8', marginTop: '2px' }}>
                    Floating glowing arrow hovering directly above your Q/W/E/R ability bar pointing to your next level-up priority.
                  </div>
                </div>
                <div
                  className={`toggle-switch ${settings.hudSkillOrder !== false ? 'on' : ''}`}
                  onClick={() => onUpdateSetting('hudSkillOrder', settings.hudSkillOrder === false ? true : false)}
                >
                  <div className="toggle-knob" />
                </div>
              </div>

              {/* 6. Core Item Goal */}
              <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', paddingBottom: '12px', borderBottom: '1px solid var(--border-subtle)' }}>
                <div>
                  <div style={{ fontSize: '13.5px', fontWeight: 700, color: '#fff', display: 'flex', alignItems: 'center', gap: '6px' }}>
                    <span>🛒 Core Item Goal & Base Gold Alert</span>
                  </div>
                  <div style={{ fontSize: '11.5px', color: '#94a3b8', marginTop: '2px' }}>
                    Tracks remaining gold needed for your next core item and notifies you when you have enough gold to recall.
                  </div>
                </div>
                <div
                  className={`toggle-switch ${settings.hudItemGoal !== false ? 'on' : ''}`}
                  onClick={() => onUpdateSetting('hudItemGoal', settings.hudItemGoal === false ? true : false)}
                >
                  <div className="toggle-knob" />
                </div>
              </div>

              {/* 7. Mayhem Augments */}
              <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', paddingBottom: '12px', borderBottom: '1px solid var(--border-subtle)' }}>
                <div>
                  <div style={{ fontSize: '13.5px', fontWeight: 700, color: '#fff', display: 'flex', alignItems: 'center', gap: '6px' }}>
                    <span>🃏 ARAM Mayhem & Arena Augment Drawer</span>
                  </div>
                  <div style={{ fontSize: '11.5px', color: '#94a3b8', marginTop: '2px' }}>
                    Provides in-game card tier recommendations (Prismatic, Gold, Silver) and optimal picks during selection rounds.
                  </div>
                </div>
                <div
                  className={`toggle-switch ${settings.hudMayhemCards !== false ? 'on' : ''}`}
                  onClick={() => onUpdateSetting('hudMayhemCards', settings.hudMayhemCards === false ? true : false)}
                >
                  <div className="toggle-knob" />
                </div>
              </div>

              {/* 8. Jungle Pathing */}
              <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between' }}>
                <div>
                  <div style={{ fontSize: '13.5px', fontWeight: 700, color: '#fff', display: 'flex', alignItems: 'center', gap: '6px' }}>
                    <span>🧭 Jungle Leash & Pathing Predictor</span>
                  </div>
                  <div style={{ fontSize: '11.5px', color: '#94a3b8', marginTop: '2px' }}>
                    Predicts level 1-4 clear pathing and alerts laners before high-risk gank windows at river scuttle respawns.
                  </div>
                </div>
                <div
                  className={`toggle-switch ${settings.hudJunglePathing !== false ? 'on' : ''}`}
                  onClick={() => onUpdateSetting('hudJunglePathing', settings.hudJunglePathing === false ? true : false)}
                >
                  <div className="toggle-knob" />
                </div>
              </div>
            </div>
          </div>
        </div>
      )}

      {/* ====================================================================== */}
      {/* TAB 3: APPEARANCE & TYPOGRAPHY                                         */}
      {/* ====================================================================== */}
      {activeTab === 'appearance' && (
        <div style={{ display: 'flex', flexDirection: 'column', gap: '16px' }}>
          <div className="glass-card">
            <div className="card-header">
              <div className="card-title">
                <Palette size={18} color="var(--blue-primary)" />
                <span>Interface Theme & Typography</span>
              </div>
              <span
                style={{
                  fontSize: '11px',
                  fontWeight: 700,
                  color: 'var(--blue-primary)',
                  background: 'rgba(0, 210, 255, 0.1)',
                  padding: '2px 8px',
                  borderRadius: '4px',
                  border: '1px solid rgba(0, 210, 255, 0.25)',
                }}
              >
                Instant Live Preview
              </span>
            </div>

            <div style={{ display: 'flex', flexDirection: 'column', gap: '14px' }}>
              <div style={{ fontSize: '12px', color: '#94a3b8' }}>
                Choose your font pairing for UI cards and tactical HUD timers.
              </div>

              <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(220px, 1fr))', gap: '10px' }}>
                {/* Option 1: Plus Jakarta Sans */}
                <button
                  onClick={() => onUpdateSetting('fontFamily', 'plus-jakarta')}
                  style={{
                    padding: '14px',
                    borderRadius: '8px',
                    background: (!settings.fontFamily || settings.fontFamily === 'plus-jakarta') ? 'rgba(0, 210, 255, 0.12)' : 'rgba(255, 255, 255, 0.02)',
                    border: (!settings.fontFamily || settings.fontFamily === 'plus-jakarta') ? '1px solid var(--blue-primary)' : '1px solid var(--border-subtle)',
                    cursor: 'pointer',
                    textAlign: 'left',
                    display: 'flex',
                    flexDirection: 'column',
                    gap: '6px',
                    transition: 'all 0.2s ease',
                  }}
                >
                  <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between' }}>
                    <span style={{ fontSize: '13px', fontWeight: 800, color: '#fff', fontFamily: 'Plus Jakarta Sans, sans-serif' }}>
                      Plus Jakarta Sans
                    </span>
                    <span style={{ fontSize: '9.5px', fontWeight: 800, color: 'var(--blue-primary)', background: 'rgba(0,210,255,0.15)', padding: '1px 6px', borderRadius: '3px' }}>
                      RECOMMENDED
                    </span>
                  </div>
                  <div style={{ fontSize: '11px', color: '#94a3b8', lineHeight: '1.4' }}>
                    Supreme legibility on dark obsidian cards with bold Rajdhani esports HUD timers.
                  </div>
                  <div style={{ fontSize: '12px', fontWeight: 800, color: '#22c55e', fontFamily: 'Rajdhani, sans-serif', letterSpacing: '0.8px', marginTop: '2px' }}>
                    Timer Preview: 05:00 • 20:00 READY
                  </div>
                </button>

                {/* Option 2: Outfit */}
                <button
                  onClick={() => onUpdateSetting('fontFamily', 'outfit')}
                  style={{
                    padding: '14px',
                    borderRadius: '8px',
                    background: settings.fontFamily === 'outfit' ? 'rgba(0, 210, 255, 0.12)' : 'rgba(255, 255, 255, 0.02)',
                    border: settings.fontFamily === 'outfit' ? '1px solid var(--blue-primary)' : '1px solid var(--border-subtle)',
                    cursor: 'pointer',
                    textAlign: 'left',
                    display: 'flex',
                    flexDirection: 'column',
                    gap: '6px',
                    transition: 'all 0.2s ease',
                  }}
                >
                  <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between' }}>
                    <span style={{ fontSize: '13px', fontWeight: 800, color: '#fff', fontFamily: 'Outfit, sans-serif' }}>
                      Outfit
                    </span>
                    <span style={{ fontSize: '9.5px', fontWeight: 800, color: '#e5b869', background: 'rgba(229,184,105,0.15)', padding: '1px 6px', borderRadius: '3px' }}>
                      FUTURISTIC
                    </span>
                  </div>
                  <div style={{ fontSize: '11px', color: '#94a3b8', lineHeight: '1.4' }}>
                    Sleek rounded geometric curves for an authentic futuristic high-tech gaming HUD.
                  </div>
                  <div style={{ fontSize: '12px', fontWeight: 800, color: '#22c55e', fontFamily: 'Rajdhani, sans-serif', letterSpacing: '0.8px', marginTop: '2px' }}>
                    Timer Preview: 05:00 • 20:00 READY
                  </div>
                </button>

                {/* Option 3: Inter */}
                <button
                  onClick={() => onUpdateSetting('fontFamily', 'inter')}
                  style={{
                    padding: '14px',
                    borderRadius: '8px',
                    background: settings.fontFamily === 'inter' ? 'rgba(0, 210, 255, 0.12)' : 'rgba(255, 255, 255, 0.02)',
                    border: settings.fontFamily === 'inter' ? '1px solid var(--blue-primary)' : '1px solid var(--border-subtle)',
                    cursor: 'pointer',
                    textAlign: 'left',
                    display: 'flex',
                    flexDirection: 'column',
                    gap: '6px',
                    transition: 'all 0.2s ease',
                  }}
                >
                  <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between' }}>
                    <span style={{ fontSize: '13px', fontWeight: 800, color: '#fff', fontFamily: 'Inter, sans-serif' }}>
                      Inter
                    </span>
                    <span style={{ fontSize: '9.5px', fontWeight: 800, color: '#94a3b8', background: 'rgba(255,255,255,0.06)', padding: '1px 6px', borderRadius: '3px' }}>
                      CLASSIC
                    </span>
                  </div>
                  <div style={{ fontSize: '11px', color: '#94a3b8', lineHeight: '1.4' }}>
                    Standard minimalist neutral interface with JetBrains Mono code-style timers.
                  </div>
                  <div style={{ fontSize: '12px', fontWeight: 800, color: '#22c55e', fontFamily: 'JetBrains Mono, monospace', letterSpacing: '0.4px', marginTop: '2px' }}>
                    Timer Preview: 05:00 • 20:00 READY
                  </div>
                </button>
              </div>
            </div>
          </div>
        </div>
      )}

      {/* ====================================================================== */}
      {/* TAB 4: SYSTEM, DIAGNOSTICS & RIOT API                                  */}
      {/* ====================================================================== */}
      {activeTab === 'system' && (
        <div style={{ display: 'flex', flexDirection: 'column', gap: '16px' }}>
          {/* Windows System Tray */}
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

              <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between' }}>
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
            </div>
          </div>

          {/* Connection Diagnostics */}
          <div className="glass-card">
            <div className="card-header">
              <div className="card-title">
                <Activity size={18} color="var(--blue-primary)" />
                <span>Connection Diagnostics</span>
              </div>
              <button className="btn-secondary" onClick={onReconnect}>
                <RotateCw size={13} />
                <span>Check Connection</span>
              </button>
            </div>

            <div style={{ display: 'flex', flexDirection: 'column', gap: '10px', fontSize: '13px' }}>
              <div style={{ display: 'flex', justifyContent: 'space-between' }}>
                <span style={{ color: '#94a3b8' }}>League Client:</span>
                <strong style={{ color: isConnected ? '#25d366' : '#ff4655' }}>
                  {isConnected ? 'Connected & Ready' : 'Offline (Open League to Connect)'}
                </strong>
              </div>
              <div style={{ display: 'flex', justifyContent: 'space-between' }}>
                <span style={{ color: '#94a3b8' }}>In-Game Match Tracker:</span>
                <strong style={{ color: '#f0e6d2' }}>Active during live matches</strong>
              </div>
              <div style={{ display: 'flex', justifyContent: 'space-between' }}>
                <span style={{ color: '#94a3b8' }}>Native Engine Architecture:</span>
                <strong style={{ color: '#0ac8b9' }}>Pure Native Rust + Windows WebView2 (~35 MB RAM)</strong>
              </div>
            </div>
          </div>

          {/* Riot Web API Integration */}
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
                {settings.riotApiKey ? '● KEY CONNECTED' : 'OPTIONAL CLOUD STATS'}
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
              <div>
                <label style={{ fontSize: '11px', fontWeight: 700, color: '#cbd5e1', textTransform: 'uppercase', marginBottom: '6px', display: 'block' }}>
                  Riot Regional Platform
                </label>
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
        </div>
      )}

      {/* ====================================================================== */}
      {/* TAB 5: UPDATES & ABOUT                                                 */}
      {/* ====================================================================== */}
      {activeTab === 'updates' && (
        <div style={{ display: 'flex', flexDirection: 'column', gap: '16px' }}>
          {/* Software Updates & Releases */}
          <div className="glass-card">
            <div className="card-header">
              <div className="card-title">
                <Sparkles size={17} color="var(--emerald-primary)" />
                <span>Software Updates & Releases</span>
              </div>
              <span
                style={{
                  fontSize: '11px',
                  fontWeight: 700,
                  color: 'var(--blue-primary)',
                  background: 'rgba(0, 210, 255, 0.1)',
                  padding: '2px 8px',
                  borderRadius: '4px',
                  border: '1px solid rgba(0, 210, 255, 0.25)',
                }}
              >
                v{appUpdate?.currentVersion || APP_VERSION}
              </span>
            </div>

            <div style={{ display: 'flex', flexDirection: 'column', gap: '12px' }}>
              <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', flexWrap: 'wrap', gap: '10px' }}>
                <div>
                  <div style={{ fontSize: '13.5px', fontWeight: 600, color: '#fff' }}>
                    GitHub Release Scanner
                  </div>
                  <div style={{ fontSize: '12px', color: '#94a3b8', marginTop: '2px' }}>
                    Scans <code style={{ color: 'var(--blue-primary)', background: 'rgba(0, 210, 255, 0.08)', padding: '2px 5px', borderRadius: '4px' }}>kotsioskl2/riftcore-companion</code> for new releases.
                  </div>
                </div>

                <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
                  {updateCheckMsg && (
                    <span style={{ fontSize: '12px', color: appUpdate?.hasUpdate ? 'var(--emerald-primary)' : '#94a3b8' }}>
                      {updateCheckMsg}
                    </span>
                  )}
                  <button
                    className="btn-secondary"
                    onClick={handleCheckUpdateClick}
                    disabled={isCheckingUpdate || isUpdatingFromSettings}
                    style={{ padding: '7px 14px', fontSize: '12px', display: 'flex', alignItems: 'center', gap: '6px', cursor: 'pointer' }}
                  >
                    {isCheckingUpdate ? <Loader2 size={13} className="spin" /> : <RotateCw size={13} />}
                    <span>{isCheckingUpdate ? 'Checking...' : 'Check for Updates'}</span>
                  </button>
                </div>
              </div>

              {appUpdate?.hasUpdate && (
                <div
                  style={{
                    marginTop: '4px',
                    padding: '12px 14px',
                    background: 'rgba(16, 185, 129, 0.08)',
                    border: '1px solid rgba(16, 185, 129, 0.25)',
                    borderRadius: '8px',
                    display: 'flex',
                    alignItems: 'center',
                    justifyContent: 'space-between',
                    flexWrap: 'wrap',
                    gap: '10px',
                  }}
                >
                  <div>
                    <div style={{ fontSize: '13px', fontWeight: 700, color: '#6ee7b7' }}>
                      🚀 Version {appUpdate.latestVersion} is ready to install!
                    </div>
                    <div style={{ fontSize: '11px', color: '#94a3b8', marginTop: '2px' }}>
                      {appUpdate.assetSize > 0 && `${(appUpdate.assetSize / (1024 * 1024)).toFixed(1)} MB · `}
                      In-place self-update with zero installer wizard required.
                    </div>
                  </div>

                  <button
                    className="btn-primary"
                    onClick={() => handleApplyUpdateClick(appUpdate.downloadUrl)}
                    disabled={isUpdatingFromSettings || !appUpdate.downloadUrl}
                    style={{
                      background: 'linear-gradient(135deg, #10b981 0%, #059669 100%)',
                      padding: '7px 16px',
                      fontSize: '12px',
                      fontWeight: 700,
                      display: 'flex',
                      alignItems: 'center',
                      gap: '6px',
                      cursor: 'pointer',
                    }}
                  >
                    {isUpdatingFromSettings ? <Loader2 size={14} className="spin" /> : <ArrowDownToLine size={14} />}
                    <span>{isUpdatingFromSettings ? 'Downloading...' : 'Update & Restart Now'}</span>
                  </button>
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
              >
                {copiedLink ? <Check size={13} /> : <Copy size={13} />}
                <span>{copiedLink ? 'Link Copied!' : 'Copy Link'}</span>
              </button>
            </div>

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

            {/* Donation Amount Tiers */}
            <div style={{ marginBottom: '16px' }}>
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

                {/* Custom Amount */}
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

              {selectedDonationAmount === 'custom' && (
                <div style={{ marginTop: '10px', display: 'flex', alignItems: 'center', gap: '8px' }}>
                  <span style={{ fontSize: '14px', color: '#fff', fontWeight: 800 }}>€</span>
                  <input
                    type="number"
                    min="1"
                    placeholder="Enter custom EUR amount"
                    value={customDonationAmount}
                    onChange={(e) => setCustomDonationAmount(e.target.value)}
                    style={{
                      flex: 1,
                      padding: '8px 12px',
                      borderRadius: '6px',
                      background: 'rgba(0, 0, 0, 0.6)',
                      border: '1px solid var(--border-subtle)',
                      color: '#fff',
                      fontSize: '13px',
                      outline: 'none',
                    }}
                  />
                </div>
              )}
            </div>

            {/* Direct QR Code & Mobile Tip Box */}
            <div
              style={{
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'space-between',
                padding: '12px 16px',
                borderRadius: '8px',
                background: 'rgba(0, 0, 0, 0.4)',
                border: '1px solid rgba(255, 255, 255, 0.06)',
                flexWrap: 'wrap',
                gap: '12px',
              }}
            >
              <div style={{ display: 'flex', alignItems: 'center', gap: '14px' }}>
                <img
                  src="./kofi_qr.png"
                  alt="Ko-fi QR Code"
                  style={{
                    width: '64px',
                    height: '64px',
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
                  <div style={{ fontSize: '11px', color: '#94a3b8', marginTop: '3px' }}>
                    Point phone camera at the QR code to tip in 3 seconds directly on mobile.
                  </div>
                </div>
              </div>

              <a
                href={KOFI_URL}
                target="_blank"
                rel="noopener noreferrer"
                style={{
                  padding: '7px 14px',
                  borderRadius: '6px',
                  background: 'rgba(0, 200, 255, 0.12)',
                  border: '1px solid var(--border-blue)',
                  color: 'var(--blue-primary)',
                  fontSize: '11.5px',
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

          {/* Riot Games Compliance Disclaimer */}
          <div
            style={{
              padding: '12px 16px',
              background: 'rgba(255, 255, 255, 0.02)',
              border: '1px solid rgba(255, 255, 255, 0.06)',
              borderRadius: '8px',
              fontSize: '11px',
              color: '#64748b',
              lineHeight: '1.5',
            }}
          >
            <div style={{ fontWeight: 700, color: '#94a3b8', marginBottom: '4px' }}>
              ⚖️ Riot Games Developer Policy Compliance
            </div>
            RiftCore Companion isn't endorsed by Riot Games and doesn't reflect the views or opinions of Riot Games or anyone officially involved in producing or managing Riot Games properties. Riot Games, and all associated properties are trademarks or registered trademarks of Riot Games, Inc.
          </div>
        </div>
      )}
    </div>
  );
};
