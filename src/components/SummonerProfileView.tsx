import React, { useState, useEffect } from 'react';
import {
  Trophy,
  RefreshCw
} from 'lucide-react';
import { SummonerProfile, RankedData, MatchHistoryItem } from '../types';
import { CHAMPIONS_LIST, getChampionIcon } from '../services/championData';
import { SUMMONER_SPELLS } from '../services/runeData';

interface SummonerProfileViewProps {
  summoner: SummonerProfile | null;
  rankedData: RankedData | null;
  onRefresh?: () => void;
}

export const SummonerProfileView: React.FC<SummonerProfileViewProps> = ({
  summoner,
  rankedData,
  onRefresh,
}) => {
  const [matches, setMatches] = useState<MatchHistoryItem[]>([]);
  const [loadingMatches, setLoadingMatches] = useState(false);

  useEffect(() => {
    async function loadMatches() {
      if (!summoner?.puuid || !window.riftAPI) return;
      setLoadingMatches(true);
      try {
        const data = await window.riftAPI.lcuRequest(
          `/lol-match-history/v1/products/lol/${summoner.puuid}/matches?begIndex=0&endIndex=10`
        );
        if (data && data.games && data.games.games) {
          const parsed = data.games.games.map((g: any, idx: number) => {
            const participantIdentity = g.participantIdentities?.find(
              (p: any) => p.player?.puuid === summoner.puuid || p.player?.summonerId === summoner.summonerId
            ) || g.participantIdentities?.[0];
            const pId = participantIdentity?.participantId || 1;
            const playerStats = g.participants?.find((p: any) => p.participantId === pId) || g.participants?.[0];
            const stats = playerStats?.stats || {};

            const win = stats.win || false;
            const durationMin = Math.max(1, g.gameDuration / 60);
            const cs = (stats.totalMinionsKilled || 0) + (stats.neutralMinionsKilled || 0);
            const csPerMin = Math.round((cs / durationMin) * 10) / 10;

            const champ = CHAMPIONS_LIST.find((c) => c.id === playerStats?.championId);

            return {
              gameId: g.gameId,
              gameDuration: g.gameDuration,
              gameCreationDate: g.gameCreationDate,
              queueId: g.queueId,
              gameMode: g.gameMode || 'CLASSIC',
              championId: playerStats?.championId || 420,
              championName: champ?.name || 'Champion',
              win,
              kills: stats.kills || 0,
              deaths: stats.deaths || 0,
              assists: stats.assists || 0,
              cs,
              csPerMin,
              goldEarned: stats.goldEarned || 0,
              damageDealt: stats.totalDamageDealtToChampions || 0,
              visionScore: stats.visionScore || 0,
              spell1Id: playerStats?.spell1Id || 4,
              spell2Id: playerStats?.spell2Id || 12,
              items: [
                stats.item0 || 0,
                stats.item1 || 0,
                stats.item2 || 0,
                stats.item3 || 0,
                stats.item4 || 0,
                stats.item5 || 0,
                stats.item6 || 0,
              ],
              badge: idx === 0 && win ? 'MVP' : win && stats.kills > 8 ? 'CARRY' : !win && stats.deaths > 7 ? 'UNLUCKY' : undefined,
            } as MatchHistoryItem;
          });
          setMatches(parsed);
        }
      } catch (err) {
        console.error('Error fetching match history:', err);
      } finally {
        setLoadingMatches(false);
      }
    }

    loadMatches();
  }, [summoner]);

  const activeSummoner = summoner || {
    gameName: 'WhiteDeathAwaits',
    tagLine: '0408',
    summonerLevel: 389,
    profileIconId: 785,
    percentCompleteForNextLevel: 72,
  };

  const soloQueue = rankedData?.queues?.find((q) => q.queueType === 'RANKED_SOLO_5x5') || {
    tier: 'BRONZE',
    division: 'I',
    leaguePoints: 55,
    wins: 3,
    losses: 2,
  };

  const winrate = Math.round((soloQueue.wins / Math.max(1, soloQueue.wins + soloQueue.losses)) * 100);

  const formatDuration = (seconds: number) => {
    const min = Math.floor(seconds / 60);
    const sec = seconds % 60;
    return `${min}m ${sec}s`;
  };

  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: '20px' }}>
      {/* Summoner Profile Banner */}
      <div
        className="glass-card"
        style={{
          padding: '24px',
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'space-between',
          flexWrap: 'wrap',
          gap: '20px',
          border: '1px solid var(--border-blue)',
        }}
      >
        <div style={{ display: 'flex', alignItems: 'center', gap: '20px' }}>
          <div style={{ position: 'relative' }}>
            <img
              src={`https://ddragon.leagueoflegends.com/cdn/14.23.1/img/profileicon/${activeSummoner.profileIconId}.png`}
              alt="Profile Icon"
              style={{
                width: '74px',
                height: '74px',
                borderRadius: '50%',
                border: '2px solid var(--blue-primary)',
                boxShadow: '0 0 16px var(--blue-glow)',
              }}
              onError={(e: any) => {
                e.target.src = 'https://ddragon.leagueoflegends.com/cdn/14.23.1/img/profileicon/29.png';
              }}
            />
            <span
              style={{
                position: 'absolute',
                bottom: '-6px',
                left: '50%',
                transform: 'translateX(-50%)',
                background: '#090e18',
                border: '1px solid var(--blue-primary)',
                color: 'var(--blue-primary)',
                fontSize: '11px',
                fontWeight: 800,
                padding: '1px 8px',
                borderRadius: '10px',
              }}
            >
              {activeSummoner.summonerLevel}
            </span>
          </div>

          <div>
            <div style={{ display: 'flex', alignItems: 'baseline', gap: '8px' }}>
              <h1 style={{ fontSize: '22px', fontWeight: 800, color: '#fff' }}>
                {activeSummoner.gameName}
              </h1>
              <span style={{ fontSize: '13px', color: '#64748b', fontWeight: 600 }}>
                #{activeSummoner.tagLine}
              </span>
            </div>

            <div style={{ display: 'flex', alignItems: 'center', gap: '14px', marginTop: '6px', fontSize: '12px', color: '#94a3b8' }}>
              <span>Region: <strong style={{ color: 'var(--blue-primary)' }}>EUNE</strong></span>
              <span>Level Progress: <strong style={{ color: '#fff' }}>{activeSummoner.percentCompleteForNextLevel}%</strong></span>
            </div>
          </div>
        </div>

        {onRefresh && (
          <button className="btn-secondary" onClick={onRefresh}>
            <RefreshCw size={13} />
            <span>Refresh</span>
          </button>
        )}
      </div>

      {/* Ranked Stats */}
      <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(280px, 1fr))', gap: '16px' }}>
        <div className="glass-card" style={{ display: 'flex', alignItems: 'center', gap: '16px' }}>
          <div
            style={{
              width: '56px',
              height: '56px',
              borderRadius: '50%',
              background: 'var(--blue-subtle)',
              border: '1px solid var(--blue-primary)',
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
              fontSize: '22px',
            }}
          >
            🛡️
          </div>
          <div style={{ flex: 1 }}>
            <div style={{ fontSize: '11px', fontWeight: 700, textTransform: 'uppercase', color: '#64748b' }}>
              Ranked Solo / Duo
            </div>
            <div style={{ display: 'flex', alignItems: 'center', gap: '8px', margin: '2px 0' }}>
              <span style={{ fontSize: '18px', fontWeight: 800, color: '#fff' }}>
                {soloQueue.tier} {soloQueue.division}
              </span>
              <span style={{ fontSize: '13px', fontWeight: 700, color: 'var(--blue-primary)' }}>
                {soloQueue.leaguePoints} LP
              </span>
            </div>
            <div style={{ fontSize: '12px', color: '#94a3b8' }}>
              <strong style={{ color: 'var(--blue-primary)' }}>{soloQueue.wins}W</strong> - <strong style={{ color: 'var(--red-primary)' }}>{soloQueue.losses}L</strong> ({winrate}% WR)
            </div>
          </div>
        </div>

        <div className="glass-card" style={{ display: 'flex', alignItems: 'center', gap: '16px' }}>
          <div
            style={{
              width: '56px',
              height: '56px',
              borderRadius: '50%',
              background: 'var(--red-subtle)',
              border: '1px solid var(--red-primary)',
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
              fontSize: '22px',
            }}
          >
            🔥
          </div>
          <div>
            <div style={{ fontSize: '11px', fontWeight: 700, textTransform: 'uppercase', color: '#64748b' }}>
              Season Performance
            </div>
            <div style={{ fontSize: '18px', fontWeight: 800, color: 'var(--red-primary)' }}>
              Positive Win Rate
            </div>
            <div style={{ fontSize: '12px', color: '#94a3b8' }}>
              Bronze I Promotion Ready
            </div>
          </div>
        </div>
      </div>

      {/* Match History List: Blue for Victory, Red for Defeat */}
      <div className="glass-card">
        <div className="card-header">
          <div className="card-title">
            <Trophy size={16} color="var(--blue-primary)" />
            <span>Match History</span>
          </div>
          <span style={{ fontSize: '11px', color: '#94a3b8' }}>
            {matches.length} Matches from League Client
          </span>
        </div>

        {loadingMatches ? (
          <div style={{ padding: '24px', textAlign: 'center', color: '#94a3b8' }}>
            Syncing matches from League Client...
          </div>
        ) : matches.length === 0 ? (
          <div style={{ padding: '24px', textAlign: 'center', color: '#94a3b8' }}>
            No recent matches found. Start a game to see stats!
          </div>
        ) : (
          <div style={{ display: 'flex', flexDirection: 'column', gap: '8px' }}>
            {matches.map((m) => {
              const kda = Math.round(((m.kills + m.assists) / Math.max(1, m.deaths)) * 10) / 10;
              return (
                <div
                  key={m.gameId}
                  style={{
                    display: 'flex',
                    alignItems: 'center',
                    justifyContent: 'space-between',
                    flexWrap: 'wrap',
                    gap: '14px',
                    padding: '12px 18px',
                    borderRadius: 'var(--radius-md)',
                    background: m.win
                      ? 'linear-gradient(90deg, rgba(0, 200, 255, 0.08) 0%, rgba(11, 18, 30, 0.6) 100%)'
                      : 'linear-gradient(90deg, rgba(255, 42, 95, 0.08) 0%, rgba(11, 18, 30, 0.6) 100%)',
                    borderLeft: `4px solid ${m.win ? 'var(--blue-primary)' : 'var(--red-primary)'}`,
                    border: '1px solid var(--border-subtle)',
                  }}
                >
                  <div style={{ minWidth: '90px' }}>
                    <div style={{ fontSize: '13px', fontWeight: 800, color: m.win ? 'var(--blue-primary)' : 'var(--red-primary)' }}>
                      {m.win ? 'VICTORY' : 'DEFEAT'}
                    </div>
                    <div style={{ fontSize: '11px', color: '#94a3b8' }}>{m.gameMode}</div>
                    <div style={{ fontSize: '10px', color: '#64748b' }}>{formatDuration(m.gameDuration)}</div>
                  </div>

                  <div style={{ display: 'flex', alignItems: 'center', gap: '10px' }}>
                    <img
                      src={getChampionIcon(m.championId)}
                      alt={m.championName}
                      style={{
                        width: '42px',
                        height: '42px',
                        borderRadius: '8px',
                        border: `1px solid ${m.win ? 'var(--blue-primary)' : 'var(--red-primary)'}`,
                      }}
                      onError={(e: any) => { e.target.src = getChampionIcon(420); }}
                    />
                    <div style={{ display: 'flex', flexDirection: 'column', gap: '3px' }}>
                      <img
                        src={SUMMONER_SPELLS[m.spell1Id]?.icon || SUMMONER_SPELLS[4].icon}
                        alt="Spell 1"
                        style={{ width: '18px', height: '18px', borderRadius: '4px' }}
                      />
                      <img
                        src={SUMMONER_SPELLS[m.spell2Id]?.icon || SUMMONER_SPELLS[12].icon}
                        alt="Spell 2"
                        style={{ width: '18px', height: '18px', borderRadius: '4px' }}
                      />
                    </div>
                  </div>

                  <div style={{ minWidth: '120px' }}>
                    <div style={{ fontSize: '14px', fontWeight: 800, color: '#fff' }}>
                      {m.kills} / <span style={{ color: 'var(--red-primary)' }}>{m.deaths}</span> / {m.assists}
                    </div>
                    <div style={{ fontSize: '11px', color: '#94a3b8' }}>
                      <strong style={{ color: 'var(--blue-primary)' }}>{kda} KDA</strong>
                    </div>
                  </div>

                  <div style={{ minWidth: '110px' }}>
                    <div style={{ fontSize: '12px', color: '#cbd5e1' }}>
                      <strong>{m.cs} CS</strong> ({m.csPerMin}/m)
                    </div>
                    <div style={{ fontSize: '11px', color: '#64748b' }}>
                      Damage: {m.damageDealt.toLocaleString()}
                    </div>
                  </div>

                  <div style={{ display: 'flex', gap: '4px' }}>
                    {m.items.map((itemId, idx) => (
                      <div
                        key={idx}
                        style={{
                          width: '26px',
                          height: '26px',
                          borderRadius: '4px',
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
                            src={`https://ddragon.leagueoflegends.com/cdn/14.23.1/img/item/${itemId}.png`}
                            alt="Item"
                            style={{ width: '100%', height: '100%' }}
                            onError={(e: any) => { e.target.style.display = 'none'; }}
                          />
                        )}
                      </div>
                    ))}
                  </div>

                  {m.badge && (
                    <div
                      style={{
                        padding: '3px 8px',
                        borderRadius: '4px',
                        fontSize: '10px',
                        fontWeight: 800,
                        background: m.badge === 'MVP' ? 'var(--blue-primary)' : 'rgba(255, 255, 255, 0.08)',
                        color: m.badge === 'MVP' ? '#06090f' : '#fff',
                      }}
                    >
                      {m.badge}
                    </div>
                  )}
                </div>
              );
            })}
          </div>
        )}
      </div>
    </div>
  );
};
