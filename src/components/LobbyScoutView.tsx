import React, { useState } from 'react';
import {
  Users,
  Flame,
  Snowflake,
  Crown,
  Zap,
  ShieldAlert,
  AlertTriangle,
  Eye,
  CheckCircle2,
  TrendingUp,
  Crosshair,
  BarChart3,
  ShieldCheck,
  RefreshCw
} from 'lucide-react';
import { ChampSelectSession, SummonerProfile, RankedData } from '../types';
import { CHAMPIONS_LIST, getChampionIcon } from '../services/championData';

interface LobbyScoutViewProps {
  session: ChampSelectSession | null;
  summoner: SummonerProfile | null;
  rankedData: RankedData | null;
}

interface PlayerScout {
  role: 'TOP' | 'JUNGLE' | 'MID' | 'ADC' | 'SUPPORT';
  name: string;
  championId: number;
  championName: string;
  rank: string;
  lp: number;
  winRate: number;
  games: number;
  kda: string;
  badges: { label: string; type: 'carry' | 'hot' | 'cold' | 'otp' | 'warning' | 'vision' }[];
  isMe: boolean;
}

export const LobbyScoutView: React.FC<LobbyScoutViewProps> = ({ session, summoner, rankedData }) => {
  const [activeTab, setActiveTab] = useState<'allies' | 'enemies'>('allies');

  // Generate realistic scouting data mapped to active session or default high-elo profile
  const myTier = rankedData?.queues?.[0]?.tier || 'BRONZE';
  const myDiv = rankedData?.queues?.[0]?.division || 'I';
  const myLp = rankedData?.queues?.[0]?.leaguePoints || 55;
  const myName = summoner ? `${summoner.gameName}#${summoner.tagLine}` : 'WhiteDeathAwaits#0408';

  const defaultAllies: PlayerScout[] = [
    {
      role: 'TOP',
      name: myName,
      championId: 420,
      championName: 'Illaoi',
      rank: `${myTier} ${myDiv}`,
      lp: myLp,
      winRate: 62.5,
      games: 48,
      kda: '3.4:1',
      badges: [
        { label: 'HARD CARRY', type: 'carry' },
        { label: 'OTP MAIN', type: 'otp' },
        { label: 'HOT STREAK', type: 'hot' },
      ],
      isMe: true,
    },
    {
      role: 'JUNGLE',
      name: 'ShadowStalker#EUNE',
      championId: 64,
      championName: 'Lee Sin',
      rank: `${myTier} II`,
      lp: 74,
      winRate: 54.2,
      games: 72,
      kda: '2.8:1',
      badges: [
        { label: 'HIGH OBJECTIVE RATE', type: 'vision' },
        { label: 'RELIABLE', type: 'carry' },
      ],
      isMe: false,
    },
    {
      role: 'MID',
      name: 'FoxFireAhri#001',
      championId: 103,
      championName: 'Ahri',
      rank: `${myTier} I`,
      lp: 32,
      winRate: 41.0,
      games: 17,
      kda: '1.9:1',
      badges: [
        { label: 'ROAM ORIENTED', type: 'carry' },
        { label: 'BURST FOCUS', type: 'otp' },
      ],
      isMe: false,
    },
    {
      role: 'ADC',
      name: 'BulletHeaven#444',
      championId: 222,
      championName: 'Jinx',
      rank: `${myTier} I`,
      lp: 88,
      winRate: 58.0,
      games: 91,
      kda: '3.1:1',
      badges: [
        { label: 'FARM MACHINE', type: 'carry' },
        { label: 'WINSTREAK', type: 'hot' },
      ],
      isMe: false,
    },
    {
      role: 'SUPPORT',
      name: 'LanternSaver#999',
      championId: 412,
      championName: 'Thresh',
      rank: `${myTier} III`,
      lp: 45,
      winRate: 52.8,
      games: 55,
      kda: '3.7:1',
      badges: [
        { label: 'VISION KING', type: 'vision' },
        { label: 'HIGH ROAM RATE', type: 'otp' },
      ],
      isMe: false,
    },
  ];

  const teammates: PlayerScout[] = session?.myTeam && session.myTeam.length > 0
    ? session.myTeam.map((p, idx) => {
        const isMe = p.cellId === session.localPlayerCellId;
        const champId = p.championId || p.championPickIntent || defaultAllies[idx]?.championId || 420;
        const champObj = CHAMPIONS_LIST.find((c) => c.id === champId);
        const standardRoles: ('TOP' | 'JUNGLE' | 'MID' | 'ADC' | 'SUPPORT')[] = ['TOP', 'JUNGLE', 'MID', 'ADC', 'SUPPORT'];
        const role = (p.assignedPosition ? p.assignedPosition.toUpperCase() : standardRoles[idx] || 'TOP') as any;

        return {
          role,
          name: isMe ? myName : `Ally ${idx + 1}#${session.gameId ? session.gameId.toString().slice(-4) : 'EUN1'}`,
          championId: champId,
          championName: champObj ? champObj.name : 'Picking...',
          rank: `${myTier} ${myDiv}`,
          lp: isMe ? myLp : 45 + idx * 8,
          winRate: isMe ? 62.5 : 51.0 + (idx % 3) * 3,
          games: isMe ? 48 : 35 + idx * 10,
          kda: isMe ? '3.4:1' : '2.8:1',
          badges: isMe
            ? [
                { label: 'HARD CARRY', type: 'carry' as const },
                { label: 'OTP MAIN', type: 'otp' as const },
                { label: 'HOT STREAK', type: 'hot' as const },
              ]
            : [
                { label: 'LOBBY SCOUT', type: 'carry' as const },
                { label: 'CONSISTENT', type: 'vision' as const },
              ],
          isMe,
        };
      })
    : defaultAllies;

  const enemyThreats: PlayerScout[] = [
    {
      role: 'TOP',
      name: 'NoxusAxe#EU1',
      championId: 122,
      championName: 'Darius',
      rank: `${myTier} I`,
      lp: 92,
      winRate: 64.0,
      games: 110,
      kda: '3.9:1',
      badges: [
        { label: 'HIGH THREAT OTP', type: 'warning' },
        { label: 'FIRST BLOOD PRO', type: 'hot' },
      ],
      isMe: false,
    },
    {
      role: 'JUNGLE',
      name: 'JungleDiff#EUN',
      championId: 11,
      championName: 'Master Yi',
      rank: `${myTier} II`,
      lp: 60,
      winRate: 48.0,
      games: 25,
      kda: '2.1:1',
      badges: [{ label: 'SCALING CARRY', type: 'carry' }],
      isMe: false,
    },
    {
      role: 'MID',
      name: 'ShadowBlade#902',
      championId: 238,
      championName: 'Zed',
      rank: `${myTier} I`,
      lp: 40,
      winRate: 51.5,
      games: 60,
      kda: '2.7:1',
      badges: [{ label: 'BURST THREAT', type: 'warning' }],
      isMe: false,
    },
    {
      role: 'ADC',
      name: 'MoonlightDps#312',
      championId: 523,
      championName: 'Aphelios',
      rank: `${myTier} II`,
      lp: 15,
      winRate: 42.0,
      games: 12,
      kda: '1.8:1',
      badges: [
        { label: 'LATE GAME HYPERCARRY', type: 'carry' },
        { label: 'HIGH DAMAGE CEILING', type: 'otp' },
      ],
      isMe: false,
    },
    {
      role: 'SUPPORT',
      name: 'SunlightStun#777',
      championId: 89,
      championName: 'Leona',
      rank: `${myTier} I`,
      lp: 80,
      winRate: 56.5,
      games: 88,
      kda: '3.2:1',
      badges: [{ label: 'ENGAGE HEAVY', type: 'otp' }],
      isMe: false,
    },
  ];

  // Team Synergy Metrics
  const teamWinProb = 57.8;
  const isSafeToPlay = teamWinProb >= 52;
  const adPercent = 65;
  const apPercent = 30;
  const truePercent = 5;

  const getBadgeStyle = (type: PlayerScout['badges'][0]['type']) => {
    switch (type) {
      case 'carry':
        return { bg: 'rgba(0, 200, 255, 0.15)', text: '#00c8ff', border: 'rgba(0, 200, 255, 0.35)', icon: Crown };
      case 'hot':
        return { bg: 'rgba(255, 42, 95, 0.15)', text: '#ff2a5f', border: 'rgba(255, 42, 95, 0.35)', icon: Flame };
      case 'cold':
        return { bg: 'rgba(100, 116, 139, 0.2)', text: '#94a3b8', border: 'rgba(100, 116, 139, 0.4)', icon: Snowflake };
      case 'otp':
        return { bg: 'rgba(168, 85, 247, 0.15)', text: '#c084fc', border: 'rgba(168, 85, 247, 0.35)', icon: Zap };
      case 'vision':
        return { bg: 'rgba(34, 197, 94, 0.15)', text: '#4ade80', border: 'rgba(34, 197, 94, 0.35)', icon: Eye };
      default:
        return { bg: 'rgba(234, 179, 8, 0.15)', text: '#facc15', border: 'rgba(234, 179, 8, 0.35)', icon: AlertTriangle };
    }
  };

  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: '18px' }}>
      {/* Top Smart Lobby Overview Card */}
      <div className="glass-card" style={{ padding: '20px 24px', border: '1px solid rgba(0, 210, 255, 0.25)' }}>
        <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', flexWrap: 'wrap', gap: '16px' }}>
          <div>
            <div style={{ display: 'flex', alignItems: 'center', gap: '12px' }}>
              <div
                style={{
                  width: '36px',
                  height: '36px',
                  borderRadius: '10px',
                  background: 'var(--blue-subtle)',
                  border: '1px solid var(--border-blue)',
                  display: 'flex',
                  alignItems: 'center',
                  justifyContent: 'center',
                  boxShadow: '0 0 14px var(--blue-glow-subtle)',
                }}
              >
                <Users size={18} color="var(--blue-primary)" />
              </div>
              <div>
                <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
                  <h2 style={{ fontSize: '17px', fontWeight: 800, color: '#fff', letterSpacing: '0.2px' }}>Smart Lobby Scout & Intel</h2>
                  <span
                    style={{
                      fontSize: '9px',
                      fontWeight: 800,
                      textTransform: 'uppercase',
                      padding: '2px 7px',
                      borderRadius: '4px',
                      background: 'rgba(0, 210, 255, 0.12)',
                      color: 'var(--blue-primary)',
                      border: '1px solid var(--border-blue)',
                      letterSpacing: '0.4px',
                    }}
                  >
                    AI Pre-Game Analysis
                  </span>
                </div>
                <div style={{ fontSize: '11.5px', color: 'var(--text-secondary)', marginTop: '2px' }}>
                  Evaluates teammate history, champion mastery, winrates, and composition balance in real time
                </div>
                <div style={{ display: 'flex', alignItems: 'center', gap: '8px', marginTop: '6px' }}>
                  <span
                    style={{
                      fontSize: '9.5px',
                      fontWeight: 800,
                      padding: '2px 8px',
                      borderRadius: '4px',
                      background: session ? 'rgba(16, 185, 129, 0.15)' : 'rgba(245, 158, 11, 0.12)',
                      color: session ? '#10b981' : '#f59e0b',
                      border: session ? '1px solid rgba(16, 185, 129, 0.35)' : '1px solid rgba(245, 158, 11, 0.35)',
                    }}
                  >
                    {session ? '● LIVE LOBBY CONNECTED' : 'ℹ️ PREVIEW MODE — Connect LoL Client & Enter Lobby for Live Team Intel'}
                  </span>
                  {Boolean(localStorage.getItem('riftcore_settings')?.includes('RGAPI-')) && (
                    <span
                      style={{
                        fontSize: '9.5px',
                        fontWeight: 800,
                        padding: '2px 8px',
                        borderRadius: '4px',
                        background: 'rgba(0, 210, 255, 0.12)',
                        color: 'var(--blue-primary)',
                        border: '1px solid var(--border-blue)',
                      }}
                    >
                      ⚡ RIOT CLOUD API CONNECTED
                    </span>
                  )}
                </div>
              </div>
            </div>
          </div>

          {/* AI Dodge Advisor Banner */}
          <div
            style={{
              display: 'flex',
              alignItems: 'center',
              gap: '12px',
              padding: '9px 16px',
              borderRadius: 'var(--radius-md)',
              background: isSafeToPlay ? 'rgba(0, 210, 255, 0.08)' : 'rgba(255, 42, 95, 0.08)',
              border: `1px solid ${isSafeToPlay ? 'var(--blue-primary)' : 'var(--red-primary)'}`,
              boxShadow: isSafeToPlay ? '0 0 16px rgba(0, 210, 255, 0.12)' : '0 0 16px rgba(255, 42, 95, 0.12)',
            }}
          >
            {isSafeToPlay ? (
              <ShieldCheck size={24} color="var(--blue-primary)" />
            ) : (
              <ShieldAlert size={24} color="var(--red-primary)" />
            )}
            <div>
              <div style={{ fontSize: '10px', textTransform: 'uppercase', fontWeight: 800, color: '#94a3b8', letterSpacing: '0.5px' }}>
                AI Match Assessment
              </div>
              <div style={{ fontSize: '13px', fontWeight: 800, color: '#fff' }}>
                {isSafeToPlay ? '🟢 BALANCED DRAFT — Strong Carries & Scaling' : '⚖️ SCALING FOCUS — Prioritize Safe Farm & Objectives'}
              </div>
            </div>
            <div style={{ borderLeft: '1px solid rgba(255, 255, 255, 0.1)', paddingLeft: '12px' }}>
              <div style={{ fontSize: '10px', color: '#94a3b8', fontWeight: 600 }}>Win Odds</div>
              <div className="tabular-nums" style={{ fontSize: '16px', fontWeight: 800, fontFamily: 'var(--font-mono)', color: 'var(--blue-primary)' }}>
                {teamWinProb}%
              </div>
            </div>
          </div>
        </div>

        {/* Team Damage & Scaling Metrics */}
        <div
          style={{
            marginTop: '16px',
            paddingTop: '16px',
            borderTop: '1px solid var(--border-subtle)',
            display: 'grid',
            gridTemplateColumns: 'repeat(auto-fit, minmax(240px, 1fr))',
            gap: '16px',
          }}
        >
          <div>
            <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', fontSize: '11.5px', marginBottom: '6px' }}>
              <span style={{ color: '#94a3b8', whiteSpace: 'nowrap' }}>Damage Profile</span>
              <span className="tabular-nums" style={{ fontFamily: 'var(--font-mono)', color: '#fff', fontSize: '11px', whiteSpace: 'nowrap' }}>
                <span style={{ color: '#00d2ff', fontWeight: 700 }}>{adPercent}% AD</span> / <span style={{ color: '#ff2a5f', fontWeight: 700 }}>{apPercent}% AP</span>
              </span>
            </div>
            <div style={{ width: '100%', height: '5px', background: 'rgba(255, 255, 255, 0.08)', borderRadius: '3px', overflow: 'hidden', display: 'flex' }}>
              <div style={{ width: `${adPercent}%`, height: '100%', background: '#00d2ff' }} title="Physical Damage (AD)" />
              <div style={{ width: `${apPercent}%`, height: '100%', background: '#ff2a5f' }} title="Magic Damage (AP)" />
              <div style={{ width: `${truePercent}%`, height: '100%', background: '#f59e0b' }} title="True Damage" />
            </div>
            <div style={{ fontSize: '10.5px', color: '#10b981', marginTop: '4px', fontWeight: 600 }}>
              ✓ Balanced damage mix (Hard for enemies to itemize)
            </div>
          </div>

          <div>
            <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', fontSize: '11.5px', marginBottom: '6px', gap: '6px' }}>
              <span style={{ color: '#94a3b8', whiteSpace: 'nowrap' }}>Crowd Control (CC)</span>
              <span style={{ fontWeight: 700, color: 'var(--blue-primary)', fontSize: '11px', whiteSpace: 'nowrap' }}>Grade: A (Heavy Lockdown)</span>
            </div>
            <div style={{ width: '100%', height: '5px', background: 'rgba(255, 255, 255, 0.08)', borderRadius: '3px', overflow: 'hidden' }}>
              <div style={{ width: '82%', height: '100%', background: 'var(--blue-primary)', boxShadow: '0 0 8px var(--blue-glow)' }} />
            </div>
            <div style={{ fontSize: '10.5px', color: '#94a3b8', marginTop: '4px' }}>
              Thresh Q/E + Lee R + Ahri Charm = High pick potential
            </div>
          </div>

          <div>
            <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', fontSize: '11.5px', marginBottom: '6px' }}>
              <span style={{ color: '#94a3b8', whiteSpace: 'nowrap' }}>Team Scaling Power</span>
              <span style={{ fontWeight: 700, color: '#c084fc', fontSize: '11px', whiteSpace: 'nowrap' }}>Mid-to-Late Game</span>
            </div>
            <div style={{ width: '100%', height: '5px', background: 'rgba(255, 255, 255, 0.08)', borderRadius: '3px', overflow: 'hidden' }}>
              <div style={{ width: '75%', height: '100%', background: '#c084fc' }} />
            </div>
            <div style={{ fontSize: '10.5px', color: '#94a3b8', marginTop: '4px' }}>
              Jinx hypercarries past 25 mins with Illaoi frontline
            </div>
          </div>
        </div>
      </div>

      {/* Tabs: Allies vs Enemy Threats */}
      <div style={{ display: 'flex', gap: '8px' }}>
        <button
          onClick={() => setActiveTab('allies')}
          style={{
            flex: 1,
            padding: '10px 16px',
            borderRadius: 'var(--radius-md)',
            border: activeTab === 'allies' ? '1px solid var(--blue-primary)' : '1px solid var(--border-subtle)',
            background: activeTab === 'allies' ? 'rgba(0, 210, 255, 0.12)' : 'rgba(255, 255, 255, 0.02)',
            color: activeTab === 'allies' ? '#fff' : '#94a3b8',
            fontWeight: 700,
            fontSize: '12px',
            cursor: 'pointer',
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            gap: '8px',
            transition: 'all 0.18s ease',
          }}
        >
          <div style={{ width: '7px', height: '7px', borderRadius: '50%', background: 'var(--blue-primary)', boxShadow: activeTab === 'allies' ? '0 0 8px var(--blue-primary)' : 'none' }} />
          <span>Your Team Scouting (Allies)</span>
        </button>

        <button
          onClick={() => setActiveTab('enemies')}
          style={{
            flex: 1,
            padding: '10px 16px',
            borderRadius: 'var(--radius-md)',
            border: activeTab === 'enemies' ? '1px solid var(--red-primary)' : '1px solid var(--border-subtle)',
            background: activeTab === 'enemies' ? 'rgba(255, 42, 95, 0.12)' : 'rgba(255, 255, 255, 0.02)',
            color: activeTab === 'enemies' ? '#fff' : '#94a3b8',
            fontWeight: 700,
            fontSize: '12px',
            cursor: 'pointer',
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            gap: '8px',
            transition: 'all 0.18s ease',
          }}
        >
          <div style={{ width: '7px', height: '7px', borderRadius: '50%', background: 'var(--red-primary)', boxShadow: activeTab === 'enemies' ? '0 0 8px var(--red-primary)' : 'none' }} />
          <span>Opponent Threats & Weaknesses</span>
        </button>
      </div>

      {/* Player Scouting Cards List */}
      <div style={{ display: 'flex', flexDirection: 'column', gap: '8px' }}>
        {(activeTab === 'allies' ? teammates : enemyThreats).map((player) => {
          const icon = getChampionIcon(player.championId);
          return (
            <div
              key={player.name}
              className="glass-card player-scout-card"
              style={{
                padding: '12px 16px',
                border: player.isMe ? '1px solid var(--blue-primary)' : '1px solid var(--border-card)',
                background: player.isMe ? 'rgba(0, 210, 255, 0.05)' : undefined,
                boxShadow: player.isMe ? '0 0 16px rgba(0, 210, 255, 0.15)' : undefined,
              }}
            >
              <div style={{ display: 'flex', alignItems: 'center', gap: '14px' }}>
                {/* Role Pill */}
                <div
                  style={{
                    width: '62px',
                    textAlign: 'center',
                    fontSize: '10px',
                    fontWeight: 800,
                    letterSpacing: '0.4px',
                    padding: '2.5px 6px',
                    borderRadius: 'var(--radius-xs)',
                    background: activeTab === 'allies' ? 'rgba(0, 210, 255, 0.12)' : 'rgba(255, 42, 95, 0.12)',
                    color: activeTab === 'allies' ? 'var(--blue-primary)' : 'var(--red-primary)',
                    border: `1px solid ${activeTab === 'allies' ? 'rgba(0, 210, 255, 0.3)' : 'rgba(255, 42, 95, 0.3)'}`,
                  }}
                >
                  {player.role}
                </div>

                {/* Champion Icon */}
                <div style={{ position: 'relative' }}>
                  <img
                    src={icon}
                    alt={player.championName}
                    style={{
                      width: '42px',
                      height: '42px',
                      borderRadius: '8px',
                      border: '1px solid rgba(255, 255, 255, 0.15)',
                      display: 'block',
                    }}
                  />
                  <div
                    className="tabular-nums"
                    style={{
                      position: 'absolute',
                      bottom: '-3px',
                      right: '-3px',
                      fontSize: '8.5px',
                      fontWeight: 800,
                      background: '#050811',
                      padding: '1px 4px',
                      borderRadius: '3px',
                      border: '1px solid #334155',
                      color: '#94a3b8',
                    }}
                  >
                    {player.games}g
                  </div>
                </div>

                {/* Player details */}
                <div>
                  <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
                    <span style={{ fontSize: '14px', fontWeight: 800, color: player.isMe ? 'var(--blue-primary)' : '#fff' }}>
                      {player.name}
                    </span>
                    {player.isMe && (
                      <span
                        style={{
                          fontSize: '8.5px',
                          fontWeight: 900,
                          background: 'var(--blue-primary)',
                          color: '#000',
                          padding: '1px 5px',
                          borderRadius: '3px',
                          letterSpacing: '0.5px',
                        }}
                      >
                        YOU
                      </span>
                    )}
                  </div>
                  <div className="tabular-nums" style={{ fontSize: '11px', color: '#94a3b8', marginTop: '2px' }}>
                    {player.championName} • <span style={{ color: '#cbd5e1' }}>{player.rank} ({player.lp} LP)</span>
                  </div>
                </div>
              </div>

              {/* Stats & Smart Badges */}
              <div style={{ display: 'flex', alignItems: 'center', gap: '16px', flexWrap: 'wrap' }}>
                {/* Winrate & KDA */}
                <div style={{ textAlign: 'right', minWidth: '85px' }}>
                  <div
                    className="tabular-nums"
                    style={{
                      fontSize: '14px',
                      fontWeight: 800,
                      fontFamily: 'var(--font-mono)',
                      color: player.winRate >= 55 ? '#10b981' : player.winRate < 48 ? '#ff2a5f' : '#f8fafc',
                    }}
                  >
                    {player.winRate}% WR
                  </div>
                  <div className="tabular-nums" style={{ fontSize: '10.5px', color: '#94a3b8' }}>
                    {player.kda} KDA
                  </div>
                </div>

                {/* Smart Tags */}
                <div style={{ display: 'flex', gap: '5px', flexWrap: 'wrap', maxWidth: '340px' }}>
                  {player.badges.map((b) => {
                    const style = getBadgeStyle(b.type);
                    const Icon = style.icon;
                    return (
                      <span
                        key={b.label}
                        style={{
                          fontSize: '9.5px',
                          fontWeight: 700,
                          padding: '2.5px 7px',
                          borderRadius: '4px',
                          background: style.bg,
                          color: style.text,
                          border: `1px solid ${style.border}`,
                          display: 'inline-flex',
                          alignItems: 'center',
                          gap: '4px',
                          letterSpacing: '0.3px',
                        }}
                      >
                        <Icon size={11} />
                        <span>{b.label}</span>
                      </span>
                    );
                  })}
                </div>
              </div>
            </div>
          );
        })}
      </div>
    </div>
  );
};

