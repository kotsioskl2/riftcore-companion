import React, { useState, useEffect } from 'react';
import {
  X,
  ChevronRight,
  ChevronLeft,
  Zap,
  ShoppingBag,
  MapPin,
  TrendingUp,
  Shield,
  Sparkles,
  Check,
  CheckCircle2,
  MousePointer,
  Keyboard,
  Compass,
} from 'lucide-react';
import { soundService } from '../services/soundService';

interface QuickTourModalProps {
  isOpen: boolean;
  onClose: () => void;
}

interface TourStep {
  title: string;
  badge: string;
  tagline: string;
  icon: React.ReactNode;
  accentColor: string;
  accentGlow: string;
  visualPreview: React.ReactNode;
  highlights: {
    icon: React.ReactNode;
    label: string;
    description: string;
  }[];
  proTip: string;
}

export const QuickTourModal: React.FC<QuickTourModalProps> = ({ isOpen, onClose }) => {
  const [currentStep, setCurrentStep] = useState(0);
  const [dontShowAgain, setDontShowAgain] = useState(false);

  useEffect(() => {
    if (isOpen) {
      setCurrentStep(0);
      try {
        const stored = localStorage.getItem('riftcore_tour_seen');
        setDontShowAgain(stored === 'true');
      } catch {}
    }
  }, [isOpen]);

  // Keyboard navigation
  useEffect(() => {
    if (!isOpen) return;
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === 'ArrowRight' || e.key === 'Enter') {
        if (currentStep < steps.length - 1) {
          handleNext();
        } else {
          handleFinish();
        }
      } else if (e.key === 'ArrowLeft' && currentStep > 0) {
        handlePrev();
      } else if (e.key === 'Escape') {
        handleFinish();
      }
    };
    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [isOpen, currentStep]);

  if (!isOpen) return null;

  const handleNext = () => {
    if (currentStep < steps.length - 1) {
      setCurrentStep((prev) => prev + 1);
      soundService.playTimerTick();
    }
  };

  const handlePrev = () => {
    if (currentStep > 0) {
      setCurrentStep((prev) => prev - 1);
      soundService.playTimerTick();
    }
  };

  const handleFinish = () => {
    try {
      localStorage.setItem('riftcore_tour_seen', 'true');
    } catch {}
    soundService.playRuneImport();
    onClose();
  };

  const steps: TourStep[] = [
    {
      title: '1-Click Auto Runes & Spells',
      badge: 'Champ Select',
      tagline: 'Instant meta setups synced directly to your League Client.',
      icon: <Zap size={22} color="#00d2ff" />,
      accentColor: '#00d2ff',
      accentGlow: 'rgba(0, 210, 255, 0.35)',
      visualPreview: (
        <div
          style={{
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            gap: '12px',
            padding: '16px',
            background: 'rgba(0, 210, 255, 0.05)',
            border: '1px solid rgba(0, 210, 255, 0.25)',
            borderRadius: '12px',
            position: 'relative',
            overflow: 'hidden',
          }}
        >
          {/* Champion Avatar */}
          <div style={{ display: 'flex', flexDirection: 'column', alignItems: 'center', gap: '4px' }}>
            <div
              style={{
                width: '46px',
                height: '46px',
                borderRadius: '10px',
                border: '2px solid #00d2ff',
                boxShadow: '0 0 14px rgba(0, 210, 255, 0.5)',
                overflow: 'hidden',
                background: '#070b14',
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'center',
              }}
            >
              <img
                src="https://ddragon.leagueoflegends.com/cdn/14.23.1/img/champion/Aatrox.png"
                alt="Locked Champ"
                style={{ width: '100%', height: '100%', objectFit: 'cover' }}
              />
            </div>
            <span style={{ fontSize: '10px', fontWeight: 800, color: '#fff' }}>Lock Pick</span>
          </div>

          {/* Sync Beam */}
          <div style={{ display: 'flex', flexDirection: 'column', alignItems: 'center', gap: '2px' }}>
            <span style={{ fontSize: '9px', fontWeight: 900, color: '#00d2ff', letterSpacing: '0.6px' }}>
              AUTO-SYNC
            </span>
            <div
              style={{
                width: '60px',
                height: '2px',
                background: 'linear-gradient(90deg, #00d2ff, #10b981)',
                boxShadow: '0 0 8px #00d2ff',
              }}
            />
            <span style={{ fontSize: '8px', color: '#94a3b8' }}>&lt;100ms</span>
          </div>

          {/* Rune Keystone + Spells Chip */}
          <div
            style={{
              display: 'flex',
              alignItems: 'center',
              gap: '6px',
              padding: '6px 10px',
              background: 'rgba(5, 12, 24, 0.85)',
              borderRadius: '8px',
              border: '1px solid rgba(16, 185, 129, 0.4)',
              boxShadow: '0 0 12px rgba(16, 185, 129, 0.25)',
            }}
          >
            <div
              style={{
                width: '28px',
                height: '28px',
                borderRadius: '50%',
                background: '#f59e0b20',
                border: '1px solid #f59e0b',
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'center',
                fontSize: '11px',
                fontWeight: 900,
                color: '#f59e0b',
              }}
            >
              ⚡
            </div>
            <div style={{ display: 'flex', flexDirection: 'column', gap: '2px' }}>
              <div style={{ fontSize: '10.5px', fontWeight: 800, color: '#10b981' }}>
                Page Injected
              </div>
              <div style={{ fontSize: '9px', color: '#94a3b8' }}>
                Conqueror + Flash/Ignite
              </div>
            </div>
          </div>
        </div>
      ),
      highlights: [
        {
          icon: <Sparkles size={14} color="#00d2ff" />,
          label: 'Zero Setup',
          description: 'Detects locked champion in real time and imports optimal perks.',
        },
        {
          icon: <MousePointer size={14} color="#00d2ff" />,
          label: 'Interactive Editor',
          description: 'Click any keystone or switch between Meta, Burst, and Anti-Tank presets.',
        },
      ],
      proTip: 'Enable "Auto-Import" at the top right to have runes injected automatically on lock-in!',
    },
    {
      title: 'LoL In-Game Shop Builds (Press "P")',
      badge: 'Item Sets',
      tagline: 'Your complete build written directly into your League shop.',
      icon: <ShoppingBag size={22} color="#10b981" />,
      accentColor: '#10b981',
      accentGlow: 'rgba(16, 185, 129, 0.35)',
      visualPreview: (
        <div
          style={{
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'space-between',
            padding: '14px 18px',
            background: 'rgba(16, 185, 129, 0.05)',
            border: '1px solid rgba(16, 185, 129, 0.25)',
            borderRadius: '12px',
          }}
        >
          {/* Keycap */}
          <div style={{ display: 'flex', alignItems: 'center', gap: '10px' }}>
            <div
              style={{
                width: '42px',
                height: '42px',
                borderRadius: '8px',
                background: 'linear-gradient(180deg, #1e293b, #0f172a)',
                border: '2px solid #10b981',
                boxShadow: '0 4px 12px rgba(16, 185, 129, 0.4), inset 0 1px 2px rgba(255,255,255,0.2)',
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'center',
                fontSize: '18px',
                fontWeight: 900,
                color: '#fff',
                fontFamily: 'monospace',
              }}
            >
              P
            </div>
            <div>
              <div style={{ fontSize: '12px', fontWeight: 800, color: '#fff' }}>Press "P" in Game</div>
              <div style={{ fontSize: '10px', color: '#94a3b8' }}>Opens RiftCore Shop Tab</div>
            </div>
          </div>

          {/* Item Progression */}
          <div style={{ display: 'flex', alignItems: 'center', gap: '4px' }}>
            <div style={{ width: '28px', height: '28px', borderRadius: '6px', background: '#3b82f625', border: '1px solid #3b82f6', display: 'flex', alignItems: 'center', justifyContent: 'center', fontSize: '9px', fontWeight: 800, color: '#fff' }}>
              1st
            </div>
            <span style={{ color: '#64748b', fontSize: '10px' }}>➔</span>
            <div style={{ width: '28px', height: '28px', borderRadius: '6px', background: '#10b98125', border: '1px solid #10b981', display: 'flex', alignItems: 'center', justifyContent: 'center', fontSize: '9px', fontWeight: 800, color: '#fff' }}>
              2nd
            </div>
            <span style={{ color: '#64748b', fontSize: '10px' }}>➔</span>
            <div style={{ width: '28px', height: '28px', borderRadius: '6px', background: '#ec489925', border: '1px solid #ec4899', display: 'flex', alignItems: 'center', justifyContent: 'center', fontSize: '9px', fontWeight: 800, color: '#fff' }}>
              3rd
            </div>
          </div>
        </div>
      ),
      highlights: [
        {
          icon: <ShoppingBag size={14} color="#10b981" />,
          label: 'Core Rush & Situational',
          description: 'Includes mythic/core rushes, counter items (Anti-Heal, Armor Shred), and elixirs.',
        },
        {
          icon: <Keyboard size={14} color="#10b981" />,
          label: 'Zero Tab-Out Needed',
          description: 'Items appear natively inside the League shop under the "RiftCore" build category.',
        },
      ],
      proTip: 'Whenever you change champions or builds, click "Push to LoL Shop" to update instantly!',
    },
    {
      title: 'Minimap Objective Pit Timers',
      badge: 'Tactical HUD',
      tagline: 'Direct pit timers for Dragon, Baron, and Voidgrubs on your minimap.',
      icon: <MapPin size={22} color="#f59e0b" />,
      accentColor: '#f59e0b',
      accentGlow: 'rgba(245, 158, 11, 0.35)',
      visualPreview: (
        <div
          style={{
            position: 'relative',
            height: '92px',
            background: 'radial-gradient(ellipse at center, rgba(15, 23, 42, 0.9) 0%, rgba(3, 7, 18, 0.98) 100%)',
            border: '1px solid rgba(245, 158, 11, 0.3)',
            borderRadius: '12px',
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'space-around',
            padding: '0 16px',
            overflow: 'hidden',
          }}
        >
          {/* Dragon Pit Badge */}
          <div
            style={{
              display: 'flex',
              alignItems: 'center',
              gap: '6px',
              padding: '6px 10px',
              background: 'rgba(239, 68, 68, 0.15)',
              border: '1px solid rgba(239, 68, 68, 0.4)',
              borderRadius: '8px',
              boxShadow: '0 0 10px rgba(239, 68, 68, 0.25)',
            }}
          >
            <span style={{ fontSize: '13px' }}>🐉</span>
            <div>
              <div style={{ fontSize: '9px', fontWeight: 800, color: '#f87171' }}>DRAGON</div>
              <div style={{ fontSize: '12px', fontWeight: 900, color: '#fff', fontFamily: 'monospace' }}>
                01:45
              </div>
            </div>
          </div>

          {/* Voidgrub / Baron Pit Badge */}
          <div
            style={{
              display: 'flex',
              alignItems: 'center',
              gap: '6px',
              padding: '6px 10px',
              background: 'rgba(168, 85, 247, 0.15)',
              border: '1px solid rgba(168, 85, 247, 0.4)',
              borderRadius: '8px',
              boxShadow: '0 0 10px rgba(168, 85, 247, 0.25)',
            }}
          >
            <span style={{ fontSize: '13px' }}>👾</span>
            <div>
              <div style={{ fontSize: '9px', fontWeight: 800, color: '#c084fc' }}>BARON / GRUBS</div>
              <div style={{ fontSize: '12px', fontWeight: 900, color: '#fff', fontFamily: 'monospace' }}>
                04:12
              </div>
            </div>
          </div>
        </div>
      ),
      highlights: [
        {
          icon: <MapPin size={14} color="#f59e0b" />,
          label: 'Placed in Pits',
          description: 'Timers sit directly inside the objective pits on your minimap, not blocking your view.',
        },
        {
          icon: <Sparkles size={14} color="#f59e0b" />,
          label: 'Respawn Alerts',
          description: 'Visual flash at 60s and 30s before spawn so your team can set up vision.',
        },
      ],
      proTip: 'Toggle "Minimap Timers" in Settings ⚙️ to switch between minimap overlay and compact mode.',
    },
    {
      title: 'Real-Time Gold Lead & CS Pace',
      badge: 'Live Game',
      tagline: 'Track team gold differential and farming pace against your rank.',
      icon: <TrendingUp size={22} color="#06b6d4" />,
      accentColor: '#06b6d4',
      accentGlow: 'rgba(6, 182, 212, 0.35)',
      visualPreview: (
        <div
          style={{
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'space-between',
            padding: '12px 16px',
            background: 'rgba(6, 182, 212, 0.05)',
            border: '1px solid rgba(6, 182, 212, 0.25)',
            borderRadius: '12px',
            gap: '12px',
          }}
        >
          {/* Gold Lead Pill */}
          <div
            style={{
              flex: 1,
              padding: '8px',
              background: 'rgba(16, 185, 129, 0.1)',
              border: '1px solid rgba(16, 185, 129, 0.35)',
              borderRadius: '8px',
              textAlign: 'center',
            }}
          >
            <div style={{ fontSize: '9px', fontWeight: 800, color: '#10b981', letterSpacing: '0.5px' }}>
              TEAM GOLD LEAD
            </div>
            <div style={{ fontSize: '15px', fontWeight: 900, color: '#fff', marginTop: '2px' }}>
              +2,450g
            </div>
          </div>

          {/* CS Pace Pill */}
          <div
            style={{
              flex: 1,
              padding: '8px',
              background: 'rgba(6, 182, 212, 0.1)',
              border: '1px solid rgba(6, 182, 212, 0.35)',
              borderRadius: '8px',
              textAlign: 'center',
            }}
          >
            <div style={{ fontSize: '9px', fontWeight: 800, color: '#06b6d4', letterSpacing: '0.5px' }}>
              CS / MIN PACE
            </div>
            <div style={{ fontSize: '15px', fontWeight: 900, color: '#fff', marginTop: '2px' }}>
              8.8 <span style={{ fontSize: '10px', color: '#10b981' }}>▲ Diamond+</span>
            </div>
          </div>
        </div>
      ),
      highlights: [
        {
          icon: <TrendingUp size={14} color="#06b6d4" />,
          label: 'Live Net Worth',
          description: 'Calculates all champion items, kills, and tower plates in real time.',
        },
        {
          icon: <Sparkles size={14} color="#06b6d4" />,
          label: 'Pacing Benchmarks',
          description: 'Compares your current CS against Emerald, Diamond, and Master rank averages.',
        },
      ],
      proTip: 'See at a glance whether you have a power spike lead to force teamfights or objectives!',
    },
    {
      title: 'Stealth Alt-Tab & Custom HUD',
      badge: 'Game Focus',
      tagline: 'Zero distractions. Auto-hides on Alt-Tab and fully modular.',
      icon: <Shield size={22} color="#a855f7" />,
      accentColor: '#a855f7',
      accentGlow: 'rgba(168, 85, 247, 0.35)',
      visualPreview: (
        <div
          style={{
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'space-around',
            padding: '14px 16px',
            background: 'rgba(168, 85, 247, 0.05)',
            border: '1px solid rgba(168, 85, 247, 0.25)',
            borderRadius: '12px',
          }}
        >
          <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
            <span
              style={{
                padding: '3px 7px',
                borderRadius: '5px',
                background: 'rgba(255, 255, 255, 0.08)',
                border: '1px solid rgba(255, 255, 255, 0.2)',
                fontSize: '11px',
                fontWeight: 800,
                color: '#fff',
                fontFamily: 'monospace',
              }}
            >
              Alt
            </span>
            <span style={{ color: '#64748b' }}>+</span>
            <span
              style={{
                padding: '3px 7px',
                borderRadius: '5px',
                background: 'rgba(255, 255, 255, 0.08)',
                border: '1px solid rgba(255, 255, 255, 0.2)',
                fontSize: '11px',
                fontWeight: 800,
                color: '#fff',
                fontFamily: 'monospace',
              }}
            >
              Tab
            </span>
            <span style={{ fontSize: '11px', color: '#94a3b8', marginLeft: '6px' }}>Auto-Hides</span>
          </div>

          <div
            style={{
              padding: '3px 8px',
              borderRadius: '6px',
              background: 'rgba(168, 85, 247, 0.15)',
              border: '1px solid rgba(168, 85, 247, 0.4)',
              fontSize: '10.5px',
              fontWeight: 800,
              color: '#d8b4fe',
            }}
          >
            ⚙️ 100% Modular
          </div>
        </div>
      ),
      highlights: [
        {
          icon: <Shield size={14} color="#a855f7" />,
          label: 'Focus Protection',
          description: 'Hides when you switch to browser or discord so your screen is always clean.',
        },
        {
          icon: <Sparkles size={14} color="#a855f7" />,
          label: 'Modular Toggles',
          description: 'Turn off gold lead, CS pace, or minimap timers individually in Settings ⚙️.',
        },
      ],
      proTip: 'Click the 📌 Pin icon in the top right titlebar to keep RiftCore always on top during draft!',
    },
  ];

  const current = steps[currentStep];

  return (
    <div
      style={{
        position: 'fixed',
        inset: 0,
        zIndex: 999999,
        background: 'rgba(2, 5, 12, 0.88)',
        backdropFilter: 'blur(20px)',
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
          maxWidth: '560px',
          background: 'linear-gradient(180deg, rgba(8, 14, 26, 0.98) 0%, rgba(4, 8, 16, 0.98) 100%)',
          border: `1px solid ${current.accentColor}55`,
          borderRadius: '16px',
          boxShadow: `0 24px 60px -8px rgba(0, 0, 0, 0.9), 0 0 35px ${current.accentGlow}`,
          padding: '24px 28px',
          display: 'flex',
          flexDirection: 'column',
          gap: '18px',
          position: 'relative',
          animation: 'modalPop 0.22s cubic-bezier(0.16, 1, 0.3, 1)',
        }}
      >
        {/* Top Header Row */}
        <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between' }}>
          <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
            <span
              style={{
                fontSize: '10px',
                fontWeight: 900,
                textTransform: 'uppercase',
                letterSpacing: '1px',
                padding: '3px 8px',
                borderRadius: '4px',
                background: `${current.accentColor}20`,
                border: `1px solid ${current.accentColor}50`,
                color: current.accentColor,
              }}
            >
              {current.badge} • Step {currentStep + 1} of {steps.length}
            </span>
          </div>

          <button
            onClick={handleFinish}
            style={{
              background: 'none',
              border: 'none',
              color: '#64748b',
              cursor: 'pointer',
              padding: '6px',
              borderRadius: '6px',
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
              transition: 'color 0.15s ease',
            }}
            title="Skip Tour"
          >
            <X size={18} />
          </button>
        </div>

        {/* Title & Tagline */}
        <div style={{ display: 'flex', alignItems: 'flex-start', gap: '14px' }}>
          <div
            style={{
              width: '46px',
              height: '46px',
              borderRadius: '12px',
              background: `${current.accentColor}18`,
              border: `1px solid ${current.accentColor}60`,
              boxShadow: `0 0 16px ${current.accentGlow}`,
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
              flexShrink: 0,
            }}
          >
            {current.icon}
          </div>
          <div>
            <h2
              style={{
                fontSize: '18px',
                fontWeight: 900,
                color: '#fff',
                letterSpacing: '0.2px',
                margin: 0,
              }}
            >
              {current.title}
            </h2>
            <p style={{ fontSize: '12.5px', color: '#94a3b8', marginTop: '3px', margin: 0, lineHeight: 1.4 }}>
              {current.tagline}
            </p>
          </div>
        </div>

        {/* Visual Graphic Demonstration */}
        {current.visualPreview}

        {/* Punchy Key Highlights */}
        <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '10px' }}>
          {current.highlights.map((h, i) => (
            <div
              key={i}
              style={{
                background: 'rgba(255, 255, 255, 0.025)',
                border: '1px solid rgba(255, 255, 255, 0.06)',
                borderRadius: '8px',
                padding: '10px 12px',
              }}
            >
              <div style={{ display: 'flex', alignItems: 'center', gap: '6px', marginBottom: '3px' }}>
                {h.icon}
                <span style={{ fontSize: '11px', fontWeight: 800, color: '#fff' }}>{h.label}</span>
              </div>
              <div style={{ fontSize: '10.5px', color: '#94a3b8', lineHeight: 1.35 }}>
                {h.description}
              </div>
            </div>
          ))}
        </div>

        {/* Pro Tip Box */}
        <div
          style={{
            padding: '8px 12px',
            borderRadius: '8px',
            background: 'rgba(255, 170, 0, 0.06)',
            border: '1px dashed rgba(255, 170, 0, 0.35)',
            display: 'flex',
            alignItems: 'center',
            gap: '8px',
          }}
        >
          <span style={{ fontSize: '11px' }}>💡</span>
          <span style={{ fontSize: '10.5px', color: '#fbbf24', fontWeight: 600 }}>
            <strong style={{ color: '#fff' }}>Pro Tip:</strong> {current.proTip}
          </span>
        </div>

        {/* Step Navigation Dots & Actions */}
        <div
          style={{
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'space-between',
            paddingTop: '8px',
            borderTop: '1px solid rgba(255, 255, 255, 0.06)',
          }}
        >
          {/* Dots Indicator */}
          <div style={{ display: 'flex', alignItems: 'center', gap: '6px' }}>
            {steps.map((_, idx) => (
              <button
                key={idx}
                onClick={() => {
                  setCurrentStep(idx);
                  soundService.playTimerTick();
                }}
                style={{
                  width: idx === currentStep ? '22px' : '7px',
                  height: '7px',
                  borderRadius: '4px',
                  background: idx === currentStep ? current.accentColor : 'rgba(255, 255, 255, 0.2)',
                  border: 'none',
                  cursor: 'pointer',
                  padding: 0,
                  transition: 'all 0.22s cubic-bezier(0.16, 1, 0.3, 1)',
                  boxShadow: idx === currentStep ? `0 0 8px ${current.accentColor}` : 'none',
                }}
                title={`Go to step ${idx + 1}`}
              />
            ))}
          </div>

          {/* Action Buttons */}
          <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
            {currentStep > 0 && (
              <button
                onClick={handlePrev}
                style={{
                  display: 'flex',
                  alignItems: 'center',
                  gap: '4px',
                  padding: '7px 14px',
                  borderRadius: '8px',
                  background: 'rgba(255, 255, 255, 0.05)',
                  border: '1px solid rgba(255, 255, 255, 0.1)',
                  color: '#cbd5e1',
                  fontSize: '11.5px',
                  fontWeight: 700,
                  cursor: 'pointer',
                  transition: 'all 0.15s ease',
                }}
              >
                <ChevronLeft size={14} />
                <span>Back</span>
              </button>
            )}

            {currentStep < steps.length - 1 ? (
              <button
                onClick={handleNext}
                style={{
                  display: 'flex',
                  alignItems: 'center',
                  gap: '6px',
                  padding: '7px 18px',
                  borderRadius: '8px',
                  background: `linear-gradient(135deg, ${current.accentColor}, #0284c7)`,
                  border: 'none',
                  color: '#000',
                  fontSize: '11.5px',
                  fontWeight: 900,
                  cursor: 'pointer',
                  boxShadow: `0 0 16px ${current.accentGlow}`,
                  transition: 'all 0.18s ease',
                }}
              >
                <span>Next</span>
                <ChevronRight size={14} />
              </button>
            ) : (
              <button
                onClick={handleFinish}
                style={{
                  display: 'flex',
                  alignItems: 'center',
                  gap: '6px',
                  padding: '7px 20px',
                  borderRadius: '8px',
                  background: 'linear-gradient(135deg, #10b981, #059669)',
                  border: 'none',
                  color: '#fff',
                  fontSize: '11.5px',
                  fontWeight: 900,
                  cursor: 'pointer',
                  boxShadow: '0 0 18px rgba(16, 185, 129, 0.45)',
                  transition: 'all 0.18s ease',
                }}
              >
                <Check size={14} />
                <span>Ready to Play!</span>
              </button>
            )}
          </div>
        </div>
      </div>
    </div>
  );
};
