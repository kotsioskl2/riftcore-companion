import React from 'react';
import {
  X,
  Trophy,
  Award,
  Crown,
  TrendingUp,
  Shield,
  Zap,
  Target,
  Sparkles,
  ChevronRight,
  Eye,
  CheckCircle2,
  AlertCircle
} from 'lucide-react';
import { MatchHistoryItem } from '../types';
import { patchService } from '../services/patchService';
import { getChampionIcon } from '../services/championData';

interface PostGameAnalysisModalProps {
  match: MatchHistoryItem | null;
  onClose: () => void;
}

export const PostGameAnalysisModal: React.FC<PostGameAnalysisModalProps> = ({ match, onClose }) => {
  if (!match) return null;

  const kda = Math.round(((match.kills + match.assists) / Math.max(1, match.deaths)) * 10) / 10;
  const isMvp = match.badge === 'MVP' || (match.win && (kda >= 3.5 || match.kills >= 10));
  const isAce = !match.win && (kda >= 2.5 || match.kills >= 7);

  // Derive Porofessor-style performance grade
  const grade = isMvp ? 'S+' : match.win && kda >= 3.0 ? 'S' : match.win ? 'A+' : isAce ? 'A' : 'B';
  const gradeColor = grade === 'S+' || grade === 'S' ? '#facc15' : grade.startsWith('A') ? 'var(--blue-primary)' : '#94a3b8';

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
          maxWidth: '580px',
          maxHeight: '92vh',
          overflowY: 'auto',
          padding: '24px',
          border: `1px solid ${match.win ? 'rgba(0, 210, 255, 0.35)' : 'rgba(255, 42, 95, 0.35)'}`,
          boxShadow: match.win
            ? '0 20px 50px rgba(0, 0, 0, 0.8), 0 0 30px rgba(0, 210, 255, 0.2)'
            : '0 20px 50px rgba(0, 0, 0, 0.8), 0 0 30px rgba(255, 42, 95, 0.2)',
          display: 'flex',
          flexDirection: 'column',
          gap: '16px',
          position: 'relative',
          borderRadius: 'var(--radius-lg)',
          animation: 'modalPop 0.2s cubic-bezier(0.16, 1, 0.3, 1)',
        }}
      >
        {/* Close Button */}
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
          }}
          title="Close Analysis"
        >
          <X size={18} />
        </button>

        {/* Header: Result + Accolade */}
        <div style={{ display: 'flex', alignItems: 'center', gap: '14px' }}>
          <img
            src={getChampionIcon(match.championId)}
            alt={match.championName}
            style={{
              width: '56px',
              height: '56px',
              borderRadius: '12px',
              border: `2px solid ${match.win ? 'var(--blue-primary)' : 'var(--red-primary)'}`,
              boxShadow: `0 0 16px ${match.win ? 'rgba(0, 210, 255, 0.3)' : 'rgba(255, 42, 95, 0.3)'}`,
            }}
            onError={(e: any) => { e.target.src = getChampionIcon(420); }}
          />
          <div>
            <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
              <span
                style={{
                  fontSize: '18px',
                  fontWeight: 900,
                  color: match.win ? 'var(--blue-primary)' : 'var(--red-primary)',
                  letterSpacing: '0.5px',
                }}
              >
                {match.win ? 'VICTORY' : 'DEFEAT'}
              </span>
              {isMvp && (
                <span
                  style={{
                    fontSize: '10px',
                    fontWeight: 900,
                    padding: '2px 7px',
                    borderRadius: '4px',
                    background: 'rgba(250, 204, 21, 0.2)',
                    color: '#facc15',
                    border: '1px solid rgba(250, 204, 21, 0.45)',
                  }}
                >
                  👑 MVP OF THE MATCH
                </span>
              )}
              {isAce && (
                <span
                  style={{
                    fontSize: '10px',
                    fontWeight: 900,
                    padding: '2px 7px',
                    borderRadius: '4px',
                    background: 'rgba(168, 85, 247, 0.2)',
                    color: '#c084fc',
                    border: '1px solid rgba(168, 85, 247, 0.45)',
                  }}
                >
                  ⭐ ACE OF DEFEAT
                </span>
              )}
            </div>
            <div style={{ fontSize: '12px', color: '#94a3b8', marginTop: '2px' }}>
              {match.championName} • {match.gameMode}
            </div>
          </div>

          <div
            style={{
              marginLeft: 'auto',
              paddingRight: '32px',
              textAlign: 'center',
            }}
          >
            <div style={{ fontSize: '9px', fontWeight: 800, textTransform: 'uppercase', color: '#64748b' }}>
              Grade
            </div>
            <div style={{ fontSize: '24px', fontWeight: 900, color: gradeColor, lineHeight: 1 }}>
              {grade}
            </div>
          </div>
        </div>

        {/* 4 Telemetry Metrics */}
        <div style={{ display: 'grid', gridTemplateColumns: 'repeat(4, 1fr)', gap: '8px' }}>
          <div style={{ background: 'rgba(255, 255, 255, 0.03)', border: '1px solid var(--border-subtle)', borderRadius: '8px', padding: '10px', textAlign: 'center' }}>
            <div style={{ fontSize: '10px', color: '#64748b', fontWeight: 700 }}>KDA RATIO</div>
            <div style={{ fontSize: '15px', fontWeight: 900, color: '#fff', marginTop: '2px' }}>
              {match.kills}/{match.deaths}/{match.assists}
            </div>
            <div style={{ fontSize: '9.5px', color: 'var(--blue-primary)', fontWeight: 700 }}>{kda} Ratio</div>
          </div>

          <div style={{ background: 'rgba(255, 255, 255, 0.03)', border: '1px solid var(--border-subtle)', borderRadius: '8px', padding: '10px', textAlign: 'center' }}>
            <div style={{ fontSize: '10px', color: '#64748b', fontWeight: 700 }}>FARM PACING</div>
            <div style={{ fontSize: '15px', fontWeight: 900, color: '#fff', marginTop: '2px' }}>
              {match.cs} CS
            </div>
            <div style={{ fontSize: '9.5px', color: match.csPerMin >= 7 ? '#10b981' : '#f59e0b', fontWeight: 700 }}>
              {match.csPerMin} CS/min
            </div>
          </div>

          <div style={{ background: 'rgba(255, 255, 255, 0.03)', border: '1px solid var(--border-subtle)', borderRadius: '8px', padding: '10px', textAlign: 'center' }}>
            <div style={{ fontSize: '10px', color: '#64748b', fontWeight: 700 }}>TOTAL DAMAGE</div>
            <div style={{ fontSize: '15px', fontWeight: 900, color: '#ff2a5f', marginTop: '2px' }}>
              {match.damageDealt.toLocaleString()}
            </div>
            <div style={{ fontSize: '9.5px', color: '#94a3b8' }}>To Champions</div>
          </div>

          <div style={{ background: 'rgba(255, 255, 255, 0.03)', border: '1px solid var(--border-subtle)', borderRadius: '8px', padding: '10px', textAlign: 'center' }}>
            <div style={{ fontSize: '10px', color: '#64748b', fontWeight: 700 }}>GOLD EARNED</div>
            <div style={{ fontSize: '15px', fontWeight: 900, color: '#facc15', marginTop: '2px' }}>
              {match.goldEarned.toLocaleString()}g
            </div>
            <div style={{ fontSize: '9.5px', color: '#94a3b8' }}>Total Net</div>
          </div>
        </div>

        {/* Porofessor-Style Tactical Performance Badges */}
        <div>
          <div style={{ fontSize: '10.5px', fontWeight: 800, textTransform: 'uppercase', color: '#64748b', marginBottom: '6px' }}>
            Match Accolades & Badges:
          </div>
          <div style={{ display: 'flex', gap: '6px', flexWrap: 'wrap' }}>
            {match.damageDealt >= 20000 && (
              <span style={{ fontSize: '10.5px', fontWeight: 700, padding: '3px 8px', borderRadius: '4px', background: 'rgba(239, 68, 68, 0.15)', color: '#f87171', border: '1px solid rgba(239, 68, 68, 0.35)' }}>
                🔥 Heavy Damage Dealer
              </span>
            )}
            {match.csPerMin >= 7.0 && (
              <span style={{ fontSize: '10.5px', fontWeight: 700, padding: '3px 8px', borderRadius: '4px', background: 'rgba(16, 185, 129, 0.15)', color: '#10b981', border: '1px solid rgba(16, 185, 129, 0.35)' }}>
                🌾 High CS Farmer
              </span>
            )}
            {match.deaths <= 3 && (
              <span style={{ fontSize: '10.5px', fontWeight: 700, padding: '3px 8px', borderRadius: '4px', background: 'rgba(0, 210, 255, 0.15)', color: '#00d2ff', border: '1px solid rgba(0, 210, 255, 0.35)' }}>
                🛡️ Low Death Count
              </span>
            )}
            {kda >= 3.5 && (
              <span style={{ fontSize: '10.5px', fontWeight: 700, padding: '3px 8px', borderRadius: '4px', background: 'rgba(250, 204, 21, 0.15)', color: '#facc15', border: '1px solid rgba(250, 204, 21, 0.35)' }}>
                👑 High KDA Impact
              </span>
            )}
            {match.visionScore >= 20 && (
              <span style={{ fontSize: '10.5px', fontWeight: 700, padding: '3px 8px', borderRadius: '4px', background: 'rgba(168, 85, 247, 0.15)', color: '#c084fc', border: '1px solid rgba(168, 85, 247, 0.35)' }}>
                👁️ Vision Ward Contributor
              </span>
            )}
          </div>
        </div>

        {/* Tactical AI Post-Game Coaching Insights */}
        <div style={{ background: 'rgba(0, 210, 255, 0.04)', border: '1px solid var(--border-blue)', borderRadius: '8px', padding: '12px', display: 'flex', flexDirection: 'column', gap: '8px' }}>
          <div style={{ fontSize: '11px', fontWeight: 800, color: 'var(--blue-primary)', display: 'flex', alignItems: 'center', gap: '5px' }}>
            <Sparkles size={13} color="var(--blue-primary)" />
            <span>AI Post-Match Tactical Coaching</span>
          </div>
          <div style={{ display: 'flex', flexDirection: 'column', gap: '6px', fontSize: '11.5px', color: '#cbd5e1', lineHeight: '1.4' }}>
            <div style={{ display: 'flex', alignItems: 'flex-start', gap: '6px' }}>
              <CheckCircle2 size={13} color="#10b981" style={{ flexShrink: 0, marginTop: '2px' }} />
              <span>
                <strong>Strengths:</strong> Maintained consistent combat output ({match.kills + match.assists} takedowns) and converted lane leads into mid-game map pressure.
              </span>
            </div>
            <div style={{ display: 'flex', alignItems: 'flex-start', gap: '6px' }}>
              <AlertCircle size={13} color="#f59e0b" style={{ flexShrink: 0, marginTop: '2px' }} />
              <span>
                <strong>Next Match Focus:</strong> Ensure vision setup in objective river pits 45 seconds prior to Dragon & Baron spawns to prevent enemy steals.
              </span>
            </div>
          </div>
        </div>

        {/* Items Used */}
        <div>
          <div style={{ fontSize: '10.5px', fontWeight: 800, textTransform: 'uppercase', color: '#64748b', marginBottom: '6px' }}>
            Final Item Build:
          </div>
          <div style={{ display: 'flex', gap: '6px' }}>
            {match.items.map((itemId, idx) => (
              <div
                key={idx}
                style={{
                  width: '32px',
                  height: '32px',
                  borderRadius: '6px',
                  background: 'rgba(0, 0, 0, 0.4)',
                  border: '1px solid rgba(255, 255, 255, 0.1)',
                  display: 'flex',
                  alignItems: 'center',
                  justifyContent: 'center',
                  overflow: 'hidden',
                }}
              >
                {itemId > 0 && (
                  <img
                    src={patchService.getItemIconUrl(itemId)}
                    alt="Item"
                    style={{ width: '100%', height: '100%' }}
                    onError={(e: any) => { e.target.style.display = 'none'; }}
                  />
                )}
              </div>
            ))}
          </div>
        </div>

        {/* Modal Action */}
        <div style={{ display: 'flex', justifyContent: 'flex-end', paddingTop: '4px' }}>
          <button
            onClick={onClose}
            className="btn-primary"
            style={{ padding: '8px 20px', fontSize: '12px', fontWeight: 800 }}
          >
            Got It
          </button>
        </div>
      </div>
    </div>
  );
};
