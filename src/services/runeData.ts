export interface RuneTree {
  id: number;
  name: string;
  icon: string;
  color: string;
  keystones: { id: number; name: string; desc: string; icon: string }[];
  slots: {
    id: number;
    name: string;
    desc: string;
    icon: string;
  }[][];
}

export const RUNE_TREES: Record<number, RuneTree> = {
  8000: {
    id: 8000,
    name: 'Precision',
    icon: 'https://ddragon.leagueoflegends.com/cdn/img/perk-images/Styles/7201_Precision.png',
    color: '#c8aa6e',
    keystones: [
      { id: 8010, name: 'Conqueror', desc: 'Gain stacking AD/AP and heal at max stacks.', icon: 'https://ddragon.leagueoflegends.com/cdn/img/perk-images/Styles/Precision/Conqueror/Conqueror.png' },
      { id: 8005, name: 'Press the Attack', desc: 'Hitting an enemy 3 times makes them vulnerable.', icon: 'https://ddragon.leagueoflegends.com/cdn/img/perk-images/Styles/Precision/PressTheAttack/PressTheAttack.png' },
      { id: 8008, name: 'Fleet Footwork', desc: 'Attacking and moving builds energy to heal and grant MS.', icon: 'https://ddragon.leagueoflegends.com/cdn/img/perk-images/Styles/Precision/FleetFootwork/FleetFootwork.png' },
      { id: 8004, name: 'Lethal Tempo', desc: 'Attacking enemy champions increases your attack speed.', icon: 'https://ddragon.leagueoflegends.com/cdn/img/perk-images/Styles/Precision/LethalTempo/LethalTempoTemp.png' }
    ],
    slots: [
      [
        { id: 9101, name: 'Absorb Life', desc: 'Killing a target heals you.', icon: 'https://ddragon.leagueoflegends.com/cdn/img/perk-images/Styles/Precision/AbsorbLife/AbsorbLife.png' },
        { id: 9111, name: 'Triumph', desc: 'Takedowns restore missing health and grant gold.', icon: 'https://ddragon.leagueoflegends.com/cdn/img/perk-images/Styles/Precision/Triumph.png' },
        { id: 8009, name: 'Presence of Mind', desc: 'Damaging enemies grants mana/energy regen.', icon: 'https://ddragon.leagueoflegends.com/cdn/img/perk-images/Styles/Precision/PresenceOfMind/PresenceOfMind.png' }
      ],
      [
        { id: 9104, name: 'Legend: Alacrity', desc: 'Gain attack speed per Legend stack.', icon: 'https://ddragon.leagueoflegends.com/cdn/img/perk-images/Styles/Precision/LegendAlacrity/LegendAlacrity.png' },
        { id: 9105, name: 'Legend: Haste', desc: 'Gain basic ability haste per Legend stack.', icon: 'https://ddragon.leagueoflegends.com/cdn/img/perk-images/Styles/Precision/LegendHaste/LegendHaste.png' },
        { id: 9103, name: 'Legend: Bloodline', desc: 'Gain lifesteal and max HP at max stacks.', icon: 'https://ddragon.leagueoflegends.com/cdn/img/perk-images/Styles/Precision/LegendBloodline/LegendBloodline.png' }
      ],
      [
        { id: 8014, name: 'Coup de Grace', desc: 'Deal more damage to low health enemies.', icon: 'https://ddragon.leagueoflegends.com/cdn/img/perk-images/Styles/Precision/CoupDeGrace/CoupDeGrace.png' },
        { id: 8017, name: 'Cut Down', desc: 'Deal more damage to high health targets.', icon: 'https://ddragon.leagueoflegends.com/cdn/img/perk-images/Styles/Precision/CutDown/CutDown.png' },
        { id: 8299, name: 'Last Stand', desc: 'Deal more damage while low on health.', icon: 'https://ddragon.leagueoflegends.com/cdn/img/perk-images/Styles/Sorcery/LastStand/LastStand.png' }
      ]
    ]
  },
  8100: {
    id: 8100,
    name: 'Domination',
    icon: 'https://ddragon.leagueoflegends.com/cdn/img/perk-images/Styles/7200_Domination.png',
    color: '#dc3545',
    keystones: [
      { id: 8112, name: 'Electrocute', desc: 'Striking a champion with 3 attacks/spells deals burst damage.', icon: 'https://ddragon.leagueoflegends.com/cdn/img/perk-images/Styles/Domination/Electrocute/Electrocute.png' },
      { id: 8128, name: 'Dark Harvest', desc: 'Damaging low health champions deals adaptive damage and harvests a soul.', icon: 'https://ddragon.leagueoflegends.com/cdn/img/perk-images/Styles/Domination/DarkHarvest/DarkHarvest.png' },
      { id: 9923, name: 'Hail of Blades', desc: 'Gain huge attack speed for 3 attacks on an enemy champion.', icon: 'https://ddragon.leagueoflegends.com/cdn/img/perk-images/Styles/Domination/HailOfBlades/HailOfBlades.png' }
    ],
    slots: [
      [
        { id: 8126, name: 'Cheap Shot', desc: 'Deal bonus true damage to impaired champions.', icon: 'https://ddragon.leagueoflegends.com/cdn/img/perk-images/Styles/Domination/CheapShot/CheapShot.png' },
        { id: 8139, name: 'Taste of Blood', desc: 'Heal when you damage an enemy champion.', icon: 'https://ddragon.leagueoflegends.com/cdn/img/perk-images/Styles/Domination/TasteOfBlood/GreenTerror_TasteOfBlood.png' },
        { id: 8143, name: 'Sudden Impact', desc: 'Gain lethality/magic pen after dash, leap, blink or stealth.', icon: 'https://ddragon.leagueoflegends.com/cdn/img/perk-images/Styles/Domination/SuddenImpact/SuddenImpact.png' }
      ],
      [
        { id: 8136, name: 'Zombie Ward', desc: 'Clearing enemy wards spawns friendly Zombie Wards.', icon: 'https://ddragon.leagueoflegends.com/cdn/img/perk-images/Styles/Domination/ZombieWard/ZombieWard.png' },
        { id: 8120, name: 'Ghost Poro', desc: 'Wards expire into Ghost Poros granting adaptive force.', icon: 'https://ddragon.leagueoflegends.com/cdn/img/perk-images/Styles/Domination/GhostPoro/GhostPoro.png' },
        { id: 8138, name: 'Eyeball Collection', desc: 'Collect eyeballs on champion takedowns for AD/AP.', icon: 'https://ddragon.leagueoflegends.com/cdn/img/perk-images/Styles/Domination/EyeballCollection/EyeballCollection.png' }
      ],
      [
        { id: 8135, name: 'Treasure Hunter', desc: 'Gain gold on unique champion takedowns.', icon: 'https://ddragon.leagueoflegends.com/cdn/img/perk-images/Styles/Domination/TreasureHunter/TreasureHunter.png' },
        { id: 8105, name: 'Relentless Hunter', desc: 'Gain out-of-combat MS on unique takedowns.', icon: 'https://ddragon.leagueoflegends.com/cdn/img/perk-images/Styles/Domination/RelentlessHunter/RelentlessHunter.png' },
        { id: 8106, name: 'Ultimate Hunter', desc: 'Your ultimate has reduced cooldown per unique takedown.', icon: 'https://ddragon.leagueoflegends.com/cdn/img/perk-images/Styles/Domination/UltimateHunter/UltimateHunter.png' }
      ]
    ]
  },
  8200: {
    id: 8200,
    name: 'Sorcery',
    icon: 'https://ddragon.leagueoflegends.com/cdn/img/perk-images/Styles/7202_Sorcery.png',
    color: '#0070ba',
    keystones: [
      { id: 8214, name: 'Summon Aery', desc: 'Attacks and abilities send Aery to damage or shield.', icon: 'https://ddragon.leagueoflegends.com/cdn/img/perk-images/Styles/Sorcery/SummonAery/SummonAery.png' },
      { id: 8229, name: 'Arcane Comet', desc: 'Damaging a champion hurls a damaging comet at their location.', icon: 'https://ddragon.leagueoflegends.com/cdn/img/perk-images/Styles/Sorcery/ArcaneComet/ArcaneComet.png' },
      { id: 8230, name: 'Phase Rush', desc: 'Hitting an enemy with 3 attacks/spells grants massive MS and slow resist.', icon: 'https://ddragon.leagueoflegends.com/cdn/img/perk-images/Styles/Sorcery/PhaseRush/PhaseRush.png' }
    ],
    slots: [
      [
        { id: 8224, name: 'Nullifying Orb', desc: 'Gain a magic shield when low on health.', icon: 'https://ddragon.leagueoflegends.com/cdn/img/perk-images/Styles/Sorcery/NullifyingOrb/Pokeshield.png' },
        { id: 8226, name: 'Manaflow Band', desc: 'Hitting enemies permanently increases max mana.', icon: 'https://ddragon.leagueoflegends.com/cdn/img/perk-images/Styles/Sorcery/ManaflowBand/ManaflowBand.png' },
        { id: 8275, name: 'Nimbus Cloak', desc: 'Casting a summoner spell grants a burst of MS.', icon: 'https://ddragon.leagueoflegends.com/cdn/img/perk-images/Styles/Sorcery/NimbusCloak/6361.png' }
      ],
      [
        { id: 8210, name: 'Transcendence', desc: 'Gain ability haste as you level up and refund cooldowns on takedowns.', icon: 'https://ddragon.leagueoflegends.com/cdn/img/perk-images/Styles/Sorcery/Transcendence/Transcendence.png' },
        { id: 8234, name: 'Celerity', desc: 'Movement speed bonuses are increased by 7%.', icon: 'https://ddragon.leagueoflegends.com/cdn/img/perk-images/Styles/Sorcery/Celerity/CelerityTemp.png' },
        { id: 8233, name: 'Absolute Focus', desc: 'Gain adaptive AD/AP while above 70% health.', icon: 'https://ddragon.leagueoflegends.com/cdn/img/perk-images/Styles/Sorcery/AbsoluteFocus/AbsoluteFocus.png' }
      ],
      [
        { id: 8237, name: 'Scorch', desc: 'First ability hit burns champion for magic damage.', icon: 'https://ddragon.leagueoflegends.com/cdn/img/perk-images/Styles/Sorcery/Scorch/Scorch.png' },
        { id: 8232, name: 'Waterwalking', desc: 'Gain MS and AP/AD while in the river.', icon: 'https://ddragon.leagueoflegends.com/cdn/img/perk-images/Styles/Sorcery/Waterwalking/Waterwalking.png' },
        { id: 8236, name: 'Gathering Storm', desc: 'Gain increasing amounts of AD/AP over time.', icon: 'https://ddragon.leagueoflegends.com/cdn/img/perk-images/Styles/Sorcery/GatheringStorm/GatheringStorm.png' }
      ]
    ]
  },
  8400: {
    id: 8400,
    name: 'Resolve',
    icon: 'https://ddragon.leagueoflegends.com/cdn/img/perk-images/Styles/7204_Resolve.png',
    color: '#28a745',
    keystones: [
      { id: 8437, name: 'Grasp of the Undying', desc: 'Periodic empowered attack deals bonus damage, heals, and increases permanent max HP.', icon: 'https://ddragon.leagueoflegends.com/cdn/img/perk-images/Styles/Resolve/GraspOfTheUndying/GraspOfTheUndying.png' },
      { id: 8439, name: 'Aftershock', desc: 'Immobilizing an enemy grants armor/MR and triggers an explosion.', icon: 'https://ddragon.leagueoflegends.com/cdn/img/perk-images/Styles/Resolve/VeteranAftershock/VeteranAftershock.png' },
      { id: 8465, name: 'Guardian', desc: 'Guard allies. If either takes damage, gain a shield and MS.', icon: 'https://ddragon.leagueoflegends.com/cdn/img/perk-images/Styles/Resolve/Guardian/Guardian.png' }
    ],
    slots: [
      [
        { id: 8446, name: 'Demolish', desc: 'Charge up a devastating strike against enemy towers.', icon: 'https://ddragon.leagueoflegends.com/cdn/img/perk-images/Styles/Resolve/Demolish/Demolish.png' },
        { id: 8463, name: 'Font of Life', desc: 'Impairing enemies marks them, healing allies that attack them.', icon: 'https://ddragon.leagueoflegends.com/cdn/img/perk-images/Styles/Resolve/FontOfLife/FontOfLife.png' },
        { id: 8401, name: 'Shield Bash', desc: 'Whenever you gain a shield, gain resistances and your next attack deals bonus damage.', icon: 'https://ddragon.leagueoflegends.com/cdn/img/perk-images/Styles/Resolve/MirrorShell/MirrorShell.png' }
      ],
      [
        { id: 8429, name: 'Conditioning', desc: 'Gain +8 Armor and MR after 12 mins and increase total resistances.', icon: 'https://ddragon.leagueoflegends.com/cdn/img/perk-images/Styles/Resolve/Conditioning/Conditioning.png' },
        { id: 8444, name: 'Second Wind', desc: 'Heal a percentage of your missing health after taking damage.', icon: 'https://ddragon.leagueoflegends.com/cdn/img/perk-images/Styles/Resolve/SecondWind/SecondWind.png' },
        { id: 8473, name: 'Bone Plating', desc: 'Next 3 enemy attacks/spells deal significantly less damage.', icon: 'https://ddragon.leagueoflegends.com/cdn/img/perk-images/Styles/Resolve/BonePlating/BonePlating.png' }
      ],
      [
        { id: 8451, name: 'Overgrowth', desc: 'Absorb life essence from dying monsters/minions for permanent HP.', icon: 'https://ddragon.leagueoflegends.com/cdn/img/perk-images/Styles/Resolve/Overgrowth/Overgrowth.png' },
        { id: 8453, name: 'Revitalize', desc: 'Heals and shields you cast or receive are 5% stronger (10% on low HP).', icon: 'https://ddragon.leagueoflegends.com/cdn/img/perk-images/Styles/Resolve/Revitalize/Revitalize.png' },
        { id: 8242, name: 'Unflinching', desc: 'Gain slow resist and tenacity while low on health.', icon: 'https://ddragon.leagueoflegends.com/cdn/img/perk-images/Styles/Sorcery/Unflinching/Unflinching.png' }
      ]
    ]
  },
  8300: {
    id: 8300,
    name: 'Inspiration',
    icon: 'https://ddragon.leagueoflegends.com/cdn/img/perk-images/Styles/7203_Whimsy.png',
    color: '#00c3ba',
    keystones: [
      { id: 8351, name: 'Glacial Augment', desc: 'Immobilizing enemy champions emits freezing rays that slow and reduce damage.', icon: 'https://ddragon.leagueoflegends.com/cdn/img/perk-images/Styles/Inspiration/GlacialAugment/GlacialAugment.png' },
      { id: 8369, name: 'First Strike', desc: 'Initiating combat against enemy champions grants bonus damage and gold.', icon: 'https://ddragon.leagueoflegends.com/cdn/img/perk-images/Styles/Inspiration/FirstStrike/FirstStrike.png' }
    ],
    slots: [
      [
        { id: 8306, name: 'Hextech Flashtraption', desc: 'Channel to blink to a new location while Flash is on cooldown.', icon: 'https://ddragon.leagueoflegends.com/cdn/img/perk-images/Styles/Inspiration/HextechFlashtraption/HextechFlashtraption.png' },
        { id: 8304, name: 'Magical Footwear', desc: 'Receive free slightly magical boots at 12 min (faster per takedown).', icon: 'https://ddragon.leagueoflegends.com/cdn/img/perk-images/Styles/Inspiration/MagicalFootwear/MagicalFootwear.png' },
        { id: 8321, name: 'Cash Back', desc: 'Receive 6% gold refund on Legendary items purchased.', icon: 'https://ddragon.leagueoflegends.com/cdn/img/perk-images/Styles/Inspiration/CashBack/CashBack2.png' }
      ],
      [
        { id: 8313, name: 'Triple Tonic', desc: 'Receive Elixirs at levels 3, 6, and 9 with bonus combat stats.', icon: 'https://ddragon.leagueoflegends.com/cdn/img/perk-images/Styles/Inspiration/PerfectTiming/AlchemistCabinet.png' },
        { id: 8352, name: 'Time Warp Tonic', desc: 'Potions grant instantaneous health restoration and MS.', icon: 'https://ddragon.leagueoflegends.com/cdn/img/perk-images/Styles/Inspiration/TimeWarpTonic/TimeWarpTonic.png' },
        { id: 8345, name: 'Biscuit Delivery', desc: 'Gain free biscuits every 2 mins that restore HP and mana.', icon: 'https://ddragon.leagueoflegends.com/cdn/img/perk-images/Styles/Inspiration/BiscuitDelivery/BiscuitDelivery.png' }
      ],
      [
        { id: 8347, name: 'Cosmic Insight', desc: '+18 Summoner Spell Haste and +10 Item Haste.', icon: 'https://ddragon.leagueoflegends.com/cdn/img/perk-images/Styles/Inspiration/CosmicInsight/CosmicInsight.png' },
        { id: 8410, name: 'Approach Velocity', desc: 'Gain bonus MS toward movement-impaired enemy champions.', icon: 'https://ddragon.leagueoflegends.com/cdn/img/perk-images/Styles/Resolve/ApproachVelocity/ApproachVelocity.png' },
        { id: 8316, name: 'Jack of All Trades', desc: 'Gain stacking stats for each different stat gained from items.', icon: 'https://ddragon.leagueoflegends.com/cdn/img/perk-images/Styles/Inspiration/JackOfAllTrades/JackofAllTrades2.png' }
      ]
    ]
  }
};

export const STAT_SHARDS: Record<number, { id: number; name: string; icon: string }> = {
  5008: { id: 5008, name: 'Adaptive Force (+9 AD/AP)', icon: 'https://ddragon.leagueoflegends.com/cdn/img/perk-images/StatMods/StatModsAdaptiveForceIcon.png' },
  5005: { id: 5005, name: 'Attack Speed (+10% AS)', icon: 'https://ddragon.leagueoflegends.com/cdn/img/perk-images/StatMods/StatModsAttackSpeedIcon.png' },
  5007: { id: 5007, name: 'Ability Haste (+8 AH)', icon: 'https://ddragon.leagueoflegends.com/cdn/img/perk-images/StatMods/StatModsCDRScalingIcon.png' },
  5010: { id: 5010, name: 'Move Speed (+2% MS)', icon: 'https://ddragon.leagueoflegends.com/cdn/img/perk-images/StatMods/StatModsMovementSpeedIcon.png' },
  5001: { id: 5001, name: 'Scaling Health (+10-180 HP)', icon: 'https://ddragon.leagueoflegends.com/cdn/img/perk-images/StatMods/StatModsHealthScalingIcon.png' },
  5011: { id: 5011, name: 'Tenacity (+Tenacity)', icon: 'https://ddragon.leagueoflegends.com/cdn/img/perk-images/StatMods/StatModsTenacityIcon.png' },
  5013: { id: 5013, name: 'Flat Health (+65 HP)', icon: 'https://ddragon.leagueoflegends.com/cdn/img/perk-images/StatMods/StatModsHealthPlusIcon.png' }
};

export const STAT_SHARD_ROWS = [
  // Row 1: Offense
  [5008, 5005, 5007],
  // Row 2: Flex
  [5008, 5010, 5001],
  // Row 3: Defense
  [5013, 5011, 5001],
];

export const SUMMONER_SPELLS: Record<number, { id: number; name: string; cooldown: number; icon: string }> = {
  4: { id: 4, name: 'Flash', cooldown: 300, icon: 'https://ddragon.leagueoflegends.com/cdn/14.23.1/img/spell/SummonerFlash.png' },
  12: { id: 12, name: 'Teleport', cooldown: 360, icon: 'https://ddragon.leagueoflegends.com/cdn/14.23.1/img/spell/SummonerTeleport.png' },
  14: { id: 14, name: 'Ignite', cooldown: 180, icon: 'https://ddragon.leagueoflegends.com/cdn/14.23.1/img/spell/SummonerDot.png' },
  11: { id: 11, name: 'Smite', cooldown: 90, icon: 'https://ddragon.leagueoflegends.com/cdn/14.23.1/img/spell/SummonerSmite.png' },
  6: { id: 6, name: 'Ghost', cooldown: 240, icon: 'https://ddragon.leagueoflegends.com/cdn/14.23.1/img/spell/SummonerHaste.png' },
  7: { id: 7, name: 'Heal', cooldown: 240, icon: 'https://ddragon.leagueoflegends.com/cdn/14.23.1/img/spell/SummonerHeal.png' },
  3: { id: 3, name: 'Exhaust', cooldown: 210, icon: 'https://ddragon.leagueoflegends.com/cdn/14.23.1/img/spell/SummonerExhaust.png' },
  21: { id: 21, name: 'Barrier', cooldown: 180, icon: 'https://ddragon.leagueoflegends.com/cdn/14.23.1/img/spell/SummonerBarrier.png' },
  1: { id: 1, name: 'Cleanse', cooldown: 210, icon: 'https://ddragon.leagueoflegends.com/cdn/14.23.1/img/spell/SummonerBoost.png' }
};
