import React from 'react';
import {
  Zap,
  Crosshair,
  Crown,
  User,
  Settings,
  Users,
  Swords,
  Layers,
  Trophy,
  Coffee,
  Sparkles
} from 'lucide-react';
import { AppSettings } from '../types';

interface SidebarProps {
  activeTab: string;
  setActiveTab: (tab: string) => void;
  settings: AppSettings;
  onUpdateSetting: (key: keyof AppSettings, value: boolean) => void;
}

export const Sidebar: React.FC<SidebarProps> = ({
  activeTab,
  setActiveTab,
  settings,
  onUpdateSetting,
}) => {
  const sections = [
    {
      category: 'Match & Live',
      items: [
        { id: 'champ-select', label: 'Champ Select', icon: Zap, badge: 'Live', badgeClass: 'badge-cyan' },
        { id: 'draft-simulator', label: 'AI Draft Coach', icon: Swords, badge: 'AI S+', badgeClass: 'badge-crimson' },
        { id: 'lobby-scout', label: 'Lobby Scout', icon: Users, badge: 'Smart', badgeClass: 'badge-emerald' },
        { id: 'live-overlay', label: 'In-Game HUD', icon: Crosshair },
      ],
    },
    {
      category: 'Meta & Builds',
      items: [
        { id: 'champion-pool', label: 'Champion Pool', icon: Layers },
        { id: 'game-modes', label: 'Arena & Modes', icon: Trophy, badge: 'Modes', badgeClass: 'badge-gold' },
        { id: 'champions', label: 'Tier List & Builds', icon: Crown },
        { id: 'profile', label: 'Profile & History', icon: User },
      ],
    },
    {
      category: 'Configuration',
      items: [
        { id: 'settings', label: 'Preferences', icon: Settings },
      ],
    },
  ];

  return (
    <aside className="sidebar" id="app-sidebar">
      <div style={{ display: 'flex', flexDirection: 'column', gap: '2px', overflowY: 'auto' }}>
        {sections.map((section, idx) => (
          <div key={section.category} style={{ marginBottom: idx < sections.length - 1 ? '6px' : '0' }}>
            <div className="sidebar-category-label">
              {section.category}
            </div>
            <div style={{ display: 'flex', flexDirection: 'column', gap: '2px' }}>
              {section.items.map((item) => {
                const Icon = item.icon;
                const isActive = activeTab === item.id;
                return (
                  <button
                    key={item.id}
                    id={`nav-${item.id}`}
                    className={`nav-item ${isActive ? 'active' : ''}`}
                    onClick={() => setActiveTab(item.id)}
                    title={item.label}
                  >
                    <Icon size={15} style={{ flexShrink: 0 }} />
                    <span style={{ flex: 1, whiteSpace: 'nowrap', overflow: 'hidden', textOverflow: 'ellipsis' }}>
                      {item.label}
                    </span>
                    {item.badge && (
                      <span className={`nav-item-badge ${item.badgeClass || 'badge-cyan'}`}>
                        {item.badge}
                      </span>
                    )}
                  </button>
                );
              })}
            </div>
          </div>
        ))}
      </div>

      <div className="quick-toggles">
        <div style={{ fontSize: '9.5px', textTransform: 'uppercase', fontWeight: 800, color: '#475569', letterSpacing: '0.8px', marginBottom: '4px' }}>
          Quick Automation
        </div>

        <div className="toggle-row">
          <span>Auto Accept Queue</span>
          <div
            id="toggle-auto-accept"
            className={`toggle-switch ${settings.autoAccept ? 'on' : ''}`}
            onClick={() => onUpdateSetting('autoAccept', !settings.autoAccept)}
            title="Automatically accepts match found ready check"
          >
            <div className="toggle-knob" />
          </div>
        </div>

        <div className="toggle-row">
          <span>Auto Import Runes</span>
          <div
            id="toggle-auto-import"
            className={`toggle-switch ${settings.autoImportRunes ? 'on' : ''}`}
            onClick={() => onUpdateSetting('autoImportRunes', !settings.autoImportRunes)}
            title="Automatically pushes best runes to client upon champion lock"
          >
            <div className="toggle-knob" />
          </div>
        </div>

        <div className="toggle-row">
          <span>Audio Alerts</span>
          <div
            id="toggle-sound-alerts"
            className={`toggle-switch ${settings.soundAlerts ? 'on' : ''}`}
            onClick={() => onUpdateSetting('soundAlerts', !settings.soundAlerts)}
            title="Plays cues for queue pop and timers"
          >
            <div className="toggle-knob" />
          </div>
        </div>

        {/* Creator Support & Fuel Link */}
        <a
          href="https://ko-fi.com/X6I629C814"
          target="_blank"
          rel="noopener noreferrer"
          style={{
            marginTop: '10px',
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            gap: '6px',
            padding: '7px 10px',
            borderRadius: 'var(--radius-sm)',
            background: 'linear-gradient(135deg, rgba(0, 210, 255, 0.12) 0%, rgba(255, 42, 95, 0.1) 100%)',
            border: '1px solid rgba(0, 210, 255, 0.25)',
            color: '#fff',
            fontSize: '11px',
            fontWeight: 700,
            textDecoration: 'none',
            transition: 'all 0.2s cubic-bezier(0.16, 1, 0.3, 1)',
            boxShadow: '0 2px 10px rgba(0, 0, 0, 0.3)',
          }}
          title="Support RiftCore on Ko-fi (Apple Pay, Google Pay & Cards — No account needed)"
        >
          <Coffee size={12} color="var(--blue-primary)" />
          <span>Fuel on Ko-fi ☕</span>
        </a>
      </div>
    </aside>
  );
};
