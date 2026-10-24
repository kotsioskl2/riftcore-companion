import React, { useState, useEffect } from 'react';
import {
  Trophy,
  RefreshCw,
  Search,
  Globe,
  Award,
  Sparkles,
  Flame,
  Shield,
  ExternalLink,
  ChevronRight,
  TrendingUp,
  UserCheck,
  CheckCircle2,
  AlertCircle
} from 'lucide-react';
import { SummonerProfile, RankedData, MatchHistoryItem } from '../types';
import { CHAMPIONS_LIST, getChampionIcon } from '../services/championData';
import { SUMMONER_SPELLS } from '../services/runeData';
import {
  riotApiService,
  RIOT_REGIONS,
  RiotChampionMastery,
  RiotLeagueEntry
} from '../services/riotApiService';

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
  // Local Matches
  const [matches, setMatches] = useState<MatchHistoryItem[]>([]);
  const [loadingMatches, setLoadingMatches] = useState(false);

  // Cloud Scout State (Riot Web API)
  const [searchRiotId, setSearchRiotId] = useState('');
  const [selectedRegion, setSelectedRegion] = useState('eun1');
  const [isCloudScout, setIsCloudScout] = useState(false);
  const [cloudLoading, setCloudLoading] = useState(false);
  const [cloudError, setCloudError] = useState<string | null>(null);

  // Cloud Summoner Details
  const [cloudAccount, setCloudAccount] = useState<{ gameName: string; tagLine: string; puuid: string } | null>(null);
  const [cloudProfile, setCloudProfile] = useState<{ summonerLevel: number; profileIconId: number; id?: string } | null>(null);
  const [cloudRanked, setCloudRanked] = useState<RiotLeagueEntry[]>([]);
  const [cloudMasteries, setCloudMasteries] = useState<RiotChampionMastery[]>([]);
  const [localSavedMatches, setLocalSavedMatches] = useState<MatchHistoryItem[]>([]);
  const [localSavedMasteries, setLocalSavedMasteries] = useState<RiotChampionMastery[]>([]);

  // Load local match history
  useEffect(() => {
    async function loadMatches() {
      if (!summoner?.puuid || !window.riftAPI || isCloudScout) return;
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
          setLocalSavedMatches(parsed);
        }
      } catch (err) {
        console.error('Error fetching match history:', err);
      } finally {
        setLoadingMatches(false);
      }
    }

    loadMatches();
  }, [summoner, isCloudScout]);

  // Load Top Champion Masteries for local summoner on mount
  useEffect(() => {
    async function loadLocalMasteries() {
      if (isCloudScout) return;
      const puuid = summoner?.puuid;
      if (!puuid) return;

      try {
        const settings = await window.riftAPI?.getSettings?.();
        const apiKey = settings?.riotApiKey || 'RGAPI-184ff7f6-57c0-4ae5-a5ce-722df7c55cc9';
        const region = settings?.riotRegion || 'eun1';
        const masteries = await riotApiService.getTopChampionMasteries(puuid, apiKey, region, 5);
        if (masteries && masteries.length > 0) {
          setCloudMasteries(masteries);
          setLocalSavedMasteries(masteries);
        }
      } catch (e) {
        console.warn('Could not load local champion masteries:', e);
      }
    }
    loadLocalMasteries();
  }, [summoner, isCloudScout]);

  // Execute Cloud Search via Riot Web API
  const handleCloudSearch = async (e?: React.FormEvent) => {
    if (e) e.preventDefault();
    if (!searchRiotId.trim()) return;

    setCloudLoading(true);
    setCloudError(null);

    let gameName = searchRiotId.trim();
    let tagLine = selectedRegion.toUpperCase();

    if (searchRiotId.includes('#')) {
      const parts = searchRiotId.split('#');
      gameName = parts[0].trim();
      tagLine = parts[1].trim();
    }

    try {
      const settings = await window.riftAPI?.getSettings?.();
      const apiKey = settings?.riotApiKey || 'RGAPI-184ff7f6-57c0-4ae5-a5ce-722df7c55cc9';

      // 1. Fetch Riot Account (Smart multi-cluster lookup across europe, americas, asia, sea)
      const accountData = await riotApiService.getAccountByRiotId(gameName, tagLine, apiKey, selectedRegion);
      if (!accountData || !accountData.puuid) {
        setCloudError(`Could not find Riot ID "${gameName}#${tagLine}". Please verify spelling and tag.`);
        setCloudLoading(false);
        return;
      }

      // Auto-detect player's active platform in cluster (e.g. eun1 vs euw1)
      const effectivePlatform = await riotApiService.resolvePlayerPlatform(
        accountData.puuid,
        accountData.cluster,
        selectedRegion,
        apiKey
      );
      if (effectivePlatform !== selectedRegion) {
        setSelectedRegion(effectivePlatform);
      }

      setCloudAccount(accountData);

      // 2. Fetch Summoner Profile (Level, ProfileIcon)
      const summonerRes = await riotApiService.getSummonerByPuuid(accountData.puuid, apiKey, effectivePlatform);
      if (summonerRes) {
        setCloudProfile(summonerRes);
      }

      // 3. Fetch Ranked Entries (Emerald, Diamond, Gold, etc.)
      const rankedRes = await riotApiService.getRankedEntries(accountData.puuid, apiKey, effectivePlatform);
      setCloudRanked(rankedRes);

      // 4. Fetch Champion Masteries
      const masteriesRes = await riotApiService.getTopChampionMasteries(accountData.puuid, apiKey, effectivePlatform, 5);
      setCloudMasteries(masteriesRes);

      // 5. Fetch Scouted Player's REAL Match History via Match-v5!
      const matchIds = await riotApiService.getRecentMatchIds(accountData.puuid, apiKey, effectivePlatform, 5);
      if (matchIds && matchIds.length > 0) {
        const detailsPromises = matchIds.slice(0, 5).map((id) => riotApiService.getMatchDetails(id, apiKey, effectivePlatform));
        const detailsList = await Promise.all(detailsPromises);
        const parsedMatches = detailsList.filter(Boolean).map((d: any, idx: number) => {
          const participant = d.info?.participants?.find((p: any) => p.puuid === accountData.puuid) || d.info?.participants?.[0];
          const stats = participant || {};
          const win = stats.win || false;
          const durationMin = Math.max(1, (d.info?.gameDuration || 1800) / 60);
          const cs = (stats.totalMinionsKilled || 0) + (stats.neutralMinionsKilled || 0);
          const csPerMin = Math.round((cs / durationMin) * 10) / 10;
          const champ = CHAMPIONS_LIST.find((c) => c.name.toLowerCase() === (stats.championName || '').toLowerCase() || c.id === stats.championId);

          return {
            gameId: d.metadata?.matchId || idx,
            gameDuration: d.info?.gameDuration || 1800,
            gameCreationDate: new Date(d.info?.gameCreation || Date.now()).toISOString(),
            queueId: d.info?.queueId || 420,
            gameMode: d.info?.gameMode || 'CLASSIC',
            championId: champ?.id || stats.championId || 420,
            championName: stats.championName || champ?.name || 'Champion',
            win,
            kills: stats.kills || 0,
            deaths: stats.deaths || 0,
            assists: stats.assists || 0,
            cs,
            csPerMin,
            goldEarned: stats.goldEarned || 0,
            damageDealt: stats.totalDamageDealtToChampions || 0,
            visionScore: stats.visionScore || 0,
            spell1Id: stats.summoner1Id || 4,
            spell2Id: stats.summoner2Id || 12,
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
        if (parsedMatches.length > 0) {
          setMatches(parsedMatches);
        }
      }

      setIsCloudScout(true);
    } catch (err: any) {
      setCloudError(err.message || 'Error connecting to Riot Games Web API.');
    } finally {
      setCloudLoading(false);
    }
  };

  const handleResetToMyProfile = () => {
    setIsCloudScout(false);
    setCloudError(null);
    setSearchRiotId('');
    if (localSavedMatches.length > 0) {
      setMatches(localSavedMatches);
    }
    if (localSavedMasteries.length > 0) {
      setCloudMasteries(localSavedMasteries);
    }
  };

  // Determine active profile data (Cloud vs Local)
  const activeSummoner = isCloudScout
    ? {
        gameName: cloudAccount?.gameName || 'Scouted Player',
        tagLine: cloudAccount?.tagLine || selectedRegion.toUpperCase(),
        summonerLevel: cloudProfile?.summonerLevel || 100,
        profileIconId: cloudProfile?.profileIconId || 29,
        percentCompleteForNextLevel: 50,
      }
    : summoner || {
        gameName: 'WhiteDeathAwaits',
        tagLine: '0408',
        summonerLevel: 389,
        profileIconId: 785,
        percentCompleteForNextLevel: 72,
      };

  const soloQueue = isCloudScout
    ? cloudRanked.find((q) => q.queueType === 'RANKED_SOLO_5x5') || {
        tier: 'UNRANKED',
        rank: '',
        leaguePoints: 0,
        wins: 0,
        losses: 0,
      }
    : rankedData?.queues?.find((q) => q.queueType === 'RANKED_SOLO_5x5') || {
        tier: 'BRONZE',
        division: 'I',
        rank: 'I',
        leaguePoints: 55,
        wins: 3,
        losses: 2,
      };

  const flexQueue = isCloudScout
    ? cloudRanked.find((q) => q.queueType === 'RANKED_FLEX_SR')
    : rankedData?.queues?.find((q) => q.queueType === 'RANKED_FLEX_SR');

  const totalSoloGames = soloQueue.wins + soloQueue.losses;
  const winrate = totalSoloGames > 0 ? Math.round((soloQueue.wins / totalSoloGames) * 100) : 0;

  const formatDuration = (seconds: number) => {
    const min = Math.floor(seconds / 60);
    const sec = seconds % 60;
    return `${min}m ${sec}s`;
  };

  // Player Performance Badges
  const badges: { label: string; color: string; bg: string }[] = [];
  if (winrate >= 60 && totalSoloGames >= 5) {
    badges.push({ label: '🔥 High Winrate', color: '#ff2a5f', bg: 'rgba(255, 42, 95, 0.15)' });
  }
  if (activeSummoner.summonerLevel > 300) {
    badges.push({ label: '⭐ Veteran Summoner', color: '#facc15', bg: 'rgba(250, 204, 21, 0.15)' });
  }
  if (cloudMasteries.length > 0 && cloudMasteries[0].championPoints > 100000) {
    badges.push({ label: '🎯 High Mastery Main', color: '#00c8ff', bg: 'rgba(0, 200, 255, 0.15)' });
  }
  if (badges.length === 0) {
    badges.push({ label: '🛡️ Ranked Competitor', color: '#22c55e', bg: 'rgba(34, 197, 94, 0.15)' });
  }

  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: '20px' }}>
      {/* CLOUD SUMMONER SCOUT SEARCH BAR (Riot Web API) */}
      <div
        className="glass-card"
        style={{
          padding: '16px 20px',
          border: '1px solid var(--border-blue)',
          background: 'linear-gradient(90deg, rgba(0, 200, 255, 0.05) 0%, rgba(11, 18, 30, 0.7) 100%)',
        }}
      >
        <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginBottom: '12px', flexWrap: 'wrap', gap: '8px' }}>
          <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
            <Search size={16} color="var(--blue-primary)" />
            <span style={{ fontSize: '13px', fontWeight: 800, color: '#fff', letterSpacing: '0.3px' }}>
              Player Search
            </span>
            <span
              style={{
                fontSize: '10px',
                fontWeight: 700,
                padding: '2px 8px',
                borderRadius: '10px',
                background: 'rgba(0, 210, 255, 0.1)',
                color: 'var(--blue-primary)',
                border: '1px solid rgba(0, 210, 255, 0.25)',
              }}
            >
              All Regions
            </span>
          </div>

          {isCloudScout && (
            <button
              onClick={handleResetToMyProfile}
              style={{
                display: 'flex',
                alignItems: 'center',
                gap: '6px',
                padding: '4px 10px',
                borderRadius: '6px',
                background: 'rgba(255, 255, 255, 0.06)',
                border: '1px solid var(--border-subtle)',
                color: '#94a3b8',
                fontSize: '11px',
                fontWeight: 700,
                cursor: 'pointer',
              }}
            >
              <UserCheck size={13} color="var(--blue-primary)" />
              <span>Back to My Client Profile</span>
            </button>
          )}
        </div>

        <form onSubmit={handleCloudSearch} style={{ display: 'flex', gap: '10px', flexWrap: 'wrap' }}>
          <div style={{ position: 'relative', flex: 1, minWidth: '220px' }}>
            <Search
              size={15}
              color="#64748b"
              style={{ position: 'absolute', left: '12px', top: '50%', transform: 'translateY(-50%)' }}
            />
            <input
              type="text"
              placeholder="Search any Riot ID (e.g. Faker#KR1, Caps#EUW)..."
              value={searchRiotId}
              onChange={(e) => setSearchRiotId(e.target.value)}
              style={{
                width: '100%',
                padding: '10px 14px 10px 36px',
                borderRadius: '8px',
                background: 'rgba(0, 0, 0, 0.4)',
                border: '1px solid var(--border-subtle)',
                color: '#fff',
                fontSize: '12px',
                outline: 'none',
              }}
            />
          </div>

          <select
            value={selectedRegion}
            onChange={(e) => setSelectedRegion(e.target.value)}
            style={{
              padding: '10px 14px',
              borderRadius: '8px',
              background: 'rgba(11, 18, 30, 0.9)',
              border: '1px solid var(--border-blue)',
              color: '#00c8ff',
              fontSize: '12px',
              fontWeight: 700,
              cursor: 'pointer',
              outline: 'none',
            }}
          >
            {RIOT_REGIONS.map((r) => (
              <option key={r.id} value={r.id}>
                {r.label}
              </option>
            ))}
          </select>

          <button
            type="submit"
            disabled={cloudLoading}
            style={{
              padding: '10px 20px',
              borderRadius: '8px',
              background: 'var(--blue-primary)',
              color: '#000',
              fontWeight: 800,
              fontSize: '12px',
              border: 'none',
              cursor: 'pointer',
              display: 'flex',
              alignItems: 'center',
              gap: '6px',
              opacity: cloudLoading ? 0.6 : 1,
            }}
          >
            {cloudLoading ? <RefreshCw size={14} className="spin" /> : <Search size={14} />}
            <span>{cloudLoading ? 'Searching...' : 'Search'}</span>
          </button>
        </form>

        {cloudError && (
          <div
            style={{
              marginTop: '10px',
              display: 'flex',
              alignItems: 'center',
              gap: '8px',
              padding: '8px 12px',
              borderRadius: '6px',
              background: 'rgba(255, 42, 95, 0.1)',
              border: '1px solid var(--border-red)',
              fontSize: '11px',
              color: 'var(--red-primary)',
            }}
          >
            <AlertCircle size={14} />
            <span>{cloudError}</span>
          </div>
        )}
      </div>

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
          border: isCloudScout ? '1px solid #c084fc' : '1px solid var(--border-blue)',
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
                border: `2px solid ${isCloudScout ? '#c084fc' : 'var(--blue-primary)'}`,
                boxShadow: `0 0 16px ${isCloudScout ? 'rgba(192, 132, 252, 0.4)' : 'var(--blue-glow)'}`,
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
                border: `1px solid ${isCloudScout ? '#c084fc' : 'var(--blue-primary)'}`,
                color: isCloudScout ? '#c084fc' : 'var(--blue-primary)',
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
            <div style={{ display: 'flex', alignItems: 'baseline', gap: '8px', flexWrap: 'wrap' }}>
              <h1 style={{ fontSize: '22px', fontWeight: 800, color: '#fff' }}>
                {activeSummoner.gameName}
              </h1>
              <span style={{ fontSize: '14px', color: '#64748b', fontWeight: 700 }}>
                #{activeSummoner.tagLine}
              </span>

              {isCloudScout ? (
                <span
                  style={{
                    fontSize: '10px',
                    fontWeight: 800,
                    padding: '2px 8px',
                    borderRadius: '12px',
                    background: 'rgba(192, 132, 252, 0.15)',
                    color: '#c084fc',
                    border: '1px solid rgba(192, 132, 252, 0.35)',
                    display: 'inline-flex',
                    alignItems: 'center',
                    gap: '4px',
                  }}
                >
                  <Globe size={10} />
                  <span>{selectedRegion.toUpperCase()}</span>
                </span>
              ) : (
                <span
                  style={{
                    fontSize: '10px',
                    fontWeight: 800,
                    padding: '2px 8px',
                    borderRadius: '12px',
                    background: 'rgba(0, 200, 255, 0.12)',
                    color: 'var(--blue-primary)',
                    border: '1px solid rgba(0, 200, 255, 0.3)',
                  }}
                >
                  My Profile
                </span>
              )}
            </div>

            {/* Badges */}
            <div style={{ display: 'flex', gap: '8px', marginTop: '8px', flexWrap: 'wrap' }}>
              {badges.map((b) => (
                <span
                  key={b.label}
                  style={{
                    fontSize: '10px',
                    fontWeight: 800,
                    padding: '3px 8px',
                    borderRadius: '5px',
                    background: b.bg,
                    color: b.color,
                    border: `1px solid ${b.color}40`,
                  }}
                >
                  {b.label}
                </span>
              ))}
            </div>
          </div>
        </div>

        {!isCloudScout && onRefresh && (
          <button className="btn-secondary" onClick={onRefresh}>
            <RefreshCw size={13} />
            <span>Sync Client</span>
          </button>
        )}
      </div>

      {/* Ranked Stats Overview */}
      <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(280px, 1fr))', gap: '16px' }}>
        {/* Solo / Duo */}
        <div className="glass-card" style={{ display: 'flex', alignItems: 'center', gap: '16px', padding: '18px 20px' }}>
          <div
            style={{
              width: '56px',
              height: '56px',
              borderRadius: '12px',
              background: 'var(--blue-subtle)',
              border: '1px solid var(--blue-primary)',
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
              fontSize: '24px',
            }}
          >
            🛡️
          </div>
          <div style={{ flex: 1 }}>
            <div style={{ fontSize: '11px', fontWeight: 800, textTransform: 'uppercase', color: '#64748b' }}>
              Ranked Solo / Duo
            </div>
            <div style={{ display: 'flex', alignItems: 'center', gap: '8px', margin: '2px 0' }}>
              <span style={{ fontSize: '18px', fontWeight: 800, color: '#fff' }}>
                {soloQueue.tier} {('division' in soloQueue ? (soloQueue as any).division : (soloQueue as any).rank) || ''}
              </span>
              <span style={{ fontSize: '13px', fontWeight: 700, color: 'var(--blue-primary)' }}>
                {soloQueue.leaguePoints} LP
              </span>
            </div>
            <div style={{ fontSize: '12px', color: '#94a3b8' }}>
              <strong style={{ color: 'var(--blue-primary)' }}>{soloQueue.wins}W</strong> -{' '}
              <strong style={{ color: 'var(--red-primary)' }}>{soloQueue.losses}L</strong> ({winrate}% WR)
            </div>
          </div>
        </div>

        {/* Flex Queue or Season Performance */}
        <div className="glass-card" style={{ display: 'flex', alignItems: 'center', gap: '16px', padding: '18px 20px' }}>
          <div
            style={{
              width: '56px',
              height: '56px',
              borderRadius: '12px',
              background: flexQueue ? 'rgba(192, 132, 252, 0.1)' : 'var(--red-subtle)',
              border: flexQueue ? '1px solid rgba(192, 132, 252, 0.3)' : '1px solid var(--red-primary)',
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
              fontSize: '24px',
            }}
          >
            {flexQueue ? '👑' : '🔥'}
          </div>
          <div>
            <div style={{ fontSize: '11px', fontWeight: 800, textTransform: 'uppercase', color: '#64748b' }}>
              {flexQueue ? 'Ranked Flex 5v5' : 'Season Momentum'}
            </div>
            <div style={{ fontSize: '18px', fontWeight: 800, color: flexQueue ? '#c084fc' : 'var(--red-primary)' }}>
              {flexQueue
                ? `${flexQueue.tier} ${(flexQueue as any).division || (flexQueue as any).rank || ''} (${flexQueue.leaguePoints} LP)`
                : winrate >= 50
                ? 'Positive Winrate'
                : 'Climbing Progression'}
            </div>
            <div style={{ fontSize: '12px', color: '#94a3b8' }}>
              {flexQueue
                ? `${flexQueue.wins}W - ${flexQueue.losses}L (${Math.round((flexQueue.wins / Math.max(1, flexQueue.wins + flexQueue.losses)) * 100)}% WR)`
                : 'Performance tracked across recent games'}
            </div>
          </div>
        </div>
      </div>

      {/* TOP CHAMPION MASTERIES (RiftCore Pro Feature) */}
      <div className="glass-card" style={{ padding: '18px 20px' }}>
        <div className="card-header" style={{ marginBottom: '14px' }}>
          <div className="card-title">
            <Award size={16} color="var(--blue-primary)" />
            <span>Top Champion Masteries</span>
          </div>
          <span style={{ fontSize: '11px', color: '#94a3b8' }}>
            Riot Games Mastery API Data
          </span>
        </div>

        {cloudMasteries.length === 0 ? (
          <div style={{ padding: '16px', textAlign: 'center', color: '#64748b', fontSize: '12px' }}>
            Champion masteries synchronizing or not yet available for this summoner.
          </div>
        ) : (
          <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(180px, 1fr))', gap: '12px' }}>
            {cloudMasteries.map((m) => {
              const champ = CHAMPIONS_LIST.find((c) => c.id === m.championId);
              const champName = champ ? champ.name : `Champion #${m.championId}`;
              const iconUrl = getChampionIcon(m.championId);

              return (
                <div
                  key={m.championId}
                  style={{
                    display: 'flex',
                    alignItems: 'center',
                    gap: '12px',
                    padding: '10px 14px',
                    borderRadius: '8px',
                    background: 'rgba(255, 255, 255, 0.03)',
                    border: '1px solid var(--border-subtle)',
                  }}
                >
                  <img
                    src={iconUrl}
                    alt={champName}
                    style={{
                      width: '42px',
                      height: '42px',
                      borderRadius: '8px',
                      border: '1px solid var(--border-blue)',
                    }}
                    onError={(e: any) => { e.target.src = getChampionIcon(420); }}
                  />
                  <div style={{ flex: 1, overflow: 'hidden' }}>
                    <div style={{ fontSize: '12px', fontWeight: 800, color: '#fff', whiteSpace: 'nowrap', overflow: 'hidden', textOverflow: 'ellipsis' }}>
                      {champName}
                    </div>
                    <div style={{ display: 'flex', alignItems: 'center', gap: '6px', marginTop: '2px' }}>
                      <span
                        style={{
                          fontSize: '9px',
                          fontWeight: 900,
                          padding: '1px 5px',
                          borderRadius: '3px',
                          background: 'rgba(250, 204, 21, 0.2)',
                          color: '#facc15',
                          border: '1px solid rgba(250, 204, 21, 0.4)',
                        }}
                      >
                        LVL {m.championLevel}
                      </span>
                      <span style={{ fontSize: '11px', color: '#94a3b8', fontFamily: 'var(--font-mono)' }}>
                        {m.championPoints.toLocaleString()} pts
                      </span>
                    </div>
                  </div>
                </div>
              );
            })}
          </div>
        )}
      </div>

      {/* Match History List: Blue for Victory, Red for Defeat */}
      <div className="glass-card">
        <div className="card-header">
          <div className="card-title">
            <Trophy size={16} color="var(--blue-primary)" />
            <span>Match History</span>
          </div>
          <span style={{ fontSize: '11px', color: '#94a3b8' }}>
            {matches.length} Matches Analyzed
          </span>
        </div>

        {loadingMatches ? (
          <div style={{ padding: '24px', textAlign: 'center', color: '#94a3b8' }}>
            Syncing matches from League Client...
          </div>
        ) : matches.length === 0 ? (
          <div style={{ padding: '24px', textAlign: 'center', color: '#94a3b8' }}>
            No recent matches found for this summoner.
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
