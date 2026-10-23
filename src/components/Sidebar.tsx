import { Zap, Crosshair, Crown, User, Settings, Volume2, CheckCircle2, Users, Swords, Layers, Snowflake } from 'lucide-react';
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
  const navItems = [
    { id: 'champ-select', label: 'Champ Select', icon: Zap, badge: 'Live' },
    { id: 'draft-simulator', label: 'AI Draft Coach', icon: Swords, badge: 'AI S+' },
    { id: 'lobby-scout', label: 'Lobby Scout', icon: Users, badge: 'Smart' },
    { id: 'live-overlay', label: 'In-Game HUD', icon: Crosshair },
    { id: 'champion-pool', label: 'Champion Pool', icon: Layers },
    { id: 'aram', label: 'ARAM Mayhem', icon: Snowflake, badge: 'Buffs' },
    { id: 'champions', label: 'Tier List & Builds', icon: Crown },
    { id: 'profile', label: 'Profile & History', icon: User },
    { id: 'settings', label: 'Preferences', icon: Settings },
  ];

  return (
    <aside className="sidebar" id="app-sidebar">
      <div style={{ display: 'flex', flexDirection: 'column', gap: '4px' }}>
        {navItems.map((item) => {
          const Icon = item.icon;
          const isActive = activeTab === item.id;
          return (
            <button
              key={item.id}
              id={`nav-${item.id}`}
              className={`nav-item ${isActive ? 'active' : ''}`}
              onClick={() => setActiveTab(item.id)}
            >
              <Icon size={16} />
              <span style={{ flex: 1 }}>{item.label}</span>
              {item.badge && (
                <span
                  style={{
                    fontSize: '9px',
                    fontWeight: 800,
                    textTransform: 'uppercase',
                    background: 'rgba(10, 200, 185, 0.2)',
                    color: '#0ac8b9',
                    padding: '1px 5px',
                    borderRadius: '4px',
                    border: '1px solid rgba(10, 200, 185, 0.4)',
                  }}
                >
                  {item.badge}
                </span>
              )}
            </button>
          );
        })}
      </div>

      <div className="quick-toggles">
        <div style={{ fontSize: '10px', textTransform: 'uppercase', fontWeight: 700, color: '#64748b', letterSpacing: '0.5px', marginBottom: '2px' }}>
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
          <span>Audio Effects</span>
          <div
            id="toggle-sound-alerts"
            className={`toggle-switch ${settings.soundAlerts ? 'on' : ''}`}
            onClick={() => onUpdateSetting('soundAlerts', !settings.soundAlerts)}
            title="Plays cues for queue pop and timers"
          >
            <div className="toggle-knob" />
          </div>
        </div>
      </div>
    </aside>
  );
};
