import { patchService } from './patchService';
import { ChampionBuild } from '../types';
import { buildChampionProfile } from './championBuildEngine';

export const CHAMPIONS_LIST: { id: number; name: string; key: string; title: string; roles: string[] }[] = [
  {
    "id": 266,
    "name": "Aatrox",
    "key": "Aatrox",
    "title": "the Darkin Blade",
    "roles": [
      "TOP",
      "JUNGLE"
    ]
  },
  {
    "id": 103,
    "name": "Ahri",
    "key": "Ahri",
    "title": "the Nine-Tailed Fox",
    "roles": [
      "MID",
      "SUPPORT"
    ]
  },
  {
    "id": 84,
    "name": "Akali",
    "key": "Akali",
    "title": "the Rogue Assassin",
    "roles": [
      "MID",
      "TOP"
    ]
  },
  {
    "id": 166,
    "name": "Akshan",
    "key": "Akshan",
    "title": "the Rogue Sentinel",
    "roles": [
      "ADC"
    ]
  },
  {
    "id": 12,
    "name": "Alistar",
    "key": "Alistar",
    "title": "the Minotaur",
    "roles": [
      "TOP",
      "SUPPORT"
    ]
  },
  {
    "id": 799,
    "name": "Ambessa",
    "key": "Ambessa",
    "title": "Matriarch of War",
    "roles": [
      "TOP",
      "JUNGLE"
    ]
  },
  {
    "id": 32,
    "name": "Amumu",
    "key": "Amumu",
    "title": "the Sad Mummy",
    "roles": [
      "JUNGLE",
      "SUPPORT"
    ]
  },
  {
    "id": 34,
    "name": "Anivia",
    "key": "Anivia",
    "title": "the Cryophoenix",
    "roles": [
      "MID",
      "SUPPORT"
    ]
  },
  {
    "id": 1,
    "name": "Annie",
    "key": "Annie",
    "title": "the Dark Child",
    "roles": [
      "MID",
      "SUPPORT"
    ]
  },
  {
    "id": 523,
    "name": "Aphelios",
    "key": "Aphelios",
    "title": "the Weapon of the Faithful",
    "roles": [
      "ADC"
    ]
  },
  {
    "id": 22,
    "name": "Ashe",
    "key": "Ashe",
    "title": "the Frost Archer",
    "roles": [
      "ADC",
      "SUPPORT"
    ]
  },
  {
    "id": 136,
    "name": "Aurelion Sol",
    "key": "AurelionSol",
    "title": "The Star Forger",
    "roles": [
      "MID",
      "SUPPORT"
    ]
  },
  {
    "id": 893,
    "name": "Aurora",
    "key": "Aurora",
    "title": "the Witch Between Worlds",
    "roles": [
      "MID",
      "SUPPORT"
    ]
  },
  {
    "id": 268,
    "name": "Azir",
    "key": "Azir",
    "title": "the Emperor of the Sands",
    "roles": [
      "ADC",
      "MID",
      "SUPPORT"
    ]
  },
  {
    "id": 432,
    "name": "Bard",
    "key": "Bard",
    "title": "the Wandering Caretaker",
    "roles": [
      "SUPPORT"
    ]
  },
  {
    "id": 200,
    "name": "Bel'Veth",
    "key": "Belveth",
    "title": "the Empress of the Void",
    "roles": [
      "TOP",
      "JUNGLE"
    ]
  },
  {
    "id": 53,
    "name": "Blitzcrank",
    "key": "Blitzcrank",
    "title": "the Great Steam Golem",
    "roles": [
      "TOP",
      "SUPPORT"
    ]
  },
  {
    "id": 63,
    "name": "Brand",
    "key": "Brand",
    "title": "the Burning Vengeance",
    "roles": [
      "MID",
      "SUPPORT"
    ]
  },
  {
    "id": 201,
    "name": "Braum",
    "key": "Braum",
    "title": "the Heart of the Freljord",
    "roles": [
      "TOP",
      "SUPPORT"
    ]
  },
  {
    "id": 233,
    "name": "Briar",
    "key": "Briar",
    "title": "the Restrained Hunger",
    "roles": [
      "TOP",
      "JUNGLE"
    ]
  },
  {
    "id": 51,
    "name": "Caitlyn",
    "key": "Caitlyn",
    "title": "the Sheriff of Piltover",
    "roles": [
      "ADC"
    ]
  },
  {
    "id": 164,
    "name": "Camille",
    "key": "Camille",
    "title": "the Steel Shadow",
    "roles": [
      "TOP",
      "JUNGLE"
    ]
  },
  {
    "id": 69,
    "name": "Cassiopeia",
    "key": "Cassiopeia",
    "title": "the Serpent's Embrace",
    "roles": [
      "MID",
      "SUPPORT"
    ]
  },
  {
    "id": 31,
    "name": "Cho'Gath",
    "key": "Chogath",
    "title": "the Terror of the Void",
    "roles": [
      "TOP",
      "SUPPORT"
    ]
  },
  {
    "id": 42,
    "name": "Corki",
    "key": "Corki",
    "title": "the Daring Bombardier",
    "roles": [
      "ADC"
    ]
  },
  {
    "id": 122,
    "name": "Darius",
    "key": "Darius",
    "title": "the Hand of Noxus",
    "roles": [
      "TOP",
      "JUNGLE"
    ]
  },
  {
    "id": 131,
    "name": "Diana",
    "key": "Diana",
    "title": "Scorn of the Moon",
    "roles": [
      "TOP",
      "JUNGLE"
    ]
  },
  {
    "id": 36,
    "name": "Dr. Mundo",
    "key": "DrMundo",
    "title": "the Madman of Zaun",
    "roles": [
      "TOP",
      "SUPPORT"
    ]
  },
  {
    "id": 119,
    "name": "Draven",
    "key": "Draven",
    "title": "the Glorious Executioner",
    "roles": [
      "ADC"
    ]
  },
  {
    "id": 245,
    "name": "Ekko",
    "key": "Ekko",
    "title": "the Boy Who Shattered Time",
    "roles": [
      "MID",
      "JUNGLE"
    ]
  },
  {
    "id": 60,
    "name": "Elise",
    "key": "Elise",
    "title": "the Spider Queen",
    "roles": [
      "MID",
      "JUNGLE"
    ]
  },
  {
    "id": 28,
    "name": "Evelynn",
    "key": "Evelynn",
    "title": "Agony's Embrace",
    "roles": [
      "MID",
      "JUNGLE"
    ]
  },
  {
    "id": 81,
    "name": "Ezreal",
    "key": "Ezreal",
    "title": "the Prodigal Explorer",
    "roles": [
      "ADC"
    ]
  },
  {
    "id": 9,
    "name": "Fiddlesticks",
    "key": "Fiddlesticks",
    "title": "the Ancient Fear",
    "roles": [
      "MID",
      "SUPPORT"
    ]
  },
  {
    "id": 114,
    "name": "Fiora",
    "key": "Fiora",
    "title": "the Grand Duelist",
    "roles": [
      "TOP",
      "JUNGLE"
    ]
  },
  {
    "id": 105,
    "name": "Fizz",
    "key": "Fizz",
    "title": "the Tidal Trickster",
    "roles": [
      "MID",
      "JUNGLE"
    ]
  },
  {
    "id": 3,
    "name": "Galio",
    "key": "Galio",
    "title": "the Colossus",
    "roles": [
      "TOP",
      "SUPPORT"
    ]
  },
  {
    "id": 41,
    "name": "Gangplank",
    "key": "Gangplank",
    "title": "the Saltwater Scourge",
    "roles": [
      "TOP",
      "JUNGLE"
    ]
  },
  {
    "id": 86,
    "name": "Garen",
    "key": "Garen",
    "title": "The Might of Demacia",
    "roles": [
      "TOP",
      "JUNGLE"
    ]
  },
  {
    "id": 150,
    "name": "Gnar",
    "key": "Gnar",
    "title": "the Missing Link",
    "roles": [
      "TOP",
      "JUNGLE"
    ]
  },
  {
    "id": 79,
    "name": "Gragas",
    "key": "Gragas",
    "title": "the Rabble Rouser",
    "roles": [
      "TOP",
      "JUNGLE"
    ]
  },
  {
    "id": 104,
    "name": "Graves",
    "key": "Graves",
    "title": "the Outlaw",
    "roles": [
      "ADC"
    ]
  },
  {
    "id": 887,
    "name": "Gwen",
    "key": "Gwen",
    "title": "The Hallowed Seamstress",
    "roles": [
      "TOP",
      "JUNGLE"
    ]
  },
  {
    "id": 120,
    "name": "Hecarim",
    "key": "Hecarim",
    "title": "the Shadow of War",
    "roles": [
      "TOP",
      "JUNGLE"
    ]
  },
  {
    "id": 74,
    "name": "Heimerdinger",
    "key": "Heimerdinger",
    "title": "the Revered Inventor",
    "roles": [
      "MID",
      "SUPPORT"
    ]
  },
  {
    "id": 910,
    "name": "Hwei",
    "key": "Hwei",
    "title": "the Visionary",
    "roles": [
      "MID",
      "SUPPORT"
    ]
  },
  {
    "id": 420,
    "name": "Illaoi",
    "key": "Illaoi",
    "title": "the Kraken Priestess",
    "roles": [
      "TOP"
    ]
  },
  {
    "id": 39,
    "name": "Irelia",
    "key": "Irelia",
    "title": "the Blade Dancer",
    "roles": [
      "TOP",
      "JUNGLE"
    ]
  },
  {
    "id": 427,
    "name": "Ivern",
    "key": "Ivern",
    "title": "the Green Father",
    "roles": [
      "JUNGLE",
      "SUPPORT"
    ]
  },
  {
    "id": 40,
    "name": "Janna",
    "key": "Janna",
    "title": "the Storm's Fury",
    "roles": [
      "SUPPORT"
    ]
  },
  {
    "id": 59,
    "name": "Jarvan IV",
    "key": "JarvanIV",
    "title": "the Exemplar of Demacia",
    "roles": [
      "TOP",
      "JUNGLE"
    ]
  },
  {
    "id": 24,
    "name": "Jax",
    "key": "Jax",
    "title": "Grandmaster at Arms",
    "roles": [
      "TOP",
      "JUNGLE"
    ]
  },
  {
    "id": 126,
    "name": "Jayce",
    "key": "Jayce",
    "title": "the Defender of Tomorrow",
    "roles": [
      "ADC",
      "TOP",
      "JUNGLE"
    ]
  },
  {
    "id": 202,
    "name": "Jhin",
    "key": "Jhin",
    "title": "the Virtuoso",
    "roles": [
      "ADC"
    ]
  },
  {
    "id": 222,
    "name": "Jinx",
    "key": "Jinx",
    "title": "the Loose Cannon",
    "roles": [
      "ADC"
    ]
  },
  {
    "id": 897,
    "name": "K'Sante",
    "key": "KSante",
    "title": "the Pride of Nazumah",
    "roles": [
      "TOP",
      "SUPPORT"
    ]
  },
  {
    "id": 145,
    "name": "Kai'Sa",
    "key": "Kaisa",
    "title": "Daughter of the Void",
    "roles": [
      "ADC"
    ]
  },
  {
    "id": 429,
    "name": "Kalista",
    "key": "Kalista",
    "title": "the Spear of Vengeance",
    "roles": [
      "ADC"
    ]
  },
  {
    "id": 43,
    "name": "Karma",
    "key": "Karma",
    "title": "the Enlightened One",
    "roles": [
      "MID",
      "SUPPORT"
    ]
  },
  {
    "id": 30,
    "name": "Karthus",
    "key": "Karthus",
    "title": "the Deathsinger",
    "roles": [
      "MID",
      "SUPPORT"
    ]
  },
  {
    "id": 38,
    "name": "Kassadin",
    "key": "Kassadin",
    "title": "the Void Walker",
    "roles": [
      "MID",
      "JUNGLE"
    ]
  },
  {
    "id": 55,
    "name": "Katarina",
    "key": "Katarina",
    "title": "the Sinister Blade",
    "roles": [
      "MID",
      "JUNGLE"
    ]
  },
  {
    "id": 10,
    "name": "Kayle",
    "key": "Kayle",
    "title": "the Righteous",
    "roles": [
      "ADC"
    ]
  },
  {
    "id": 141,
    "name": "Kayn",
    "key": "Kayn",
    "title": "the Shadow Reaper",
    "roles": [
      "TOP",
      "JUNGLE"
    ]
  },
  {
    "id": 85,
    "name": "Kennen",
    "key": "Kennen",
    "title": "the Heart of the Tempest",
    "roles": [
      "MID",
      "SUPPORT"
    ]
  },
  {
    "id": 121,
    "name": "Kha'Zix",
    "key": "Khazix",
    "title": "the Voidreaver",
    "roles": [
      "MID",
      "JUNGLE"
    ]
  },
  {
    "id": 203,
    "name": "Kindred",
    "key": "Kindred",
    "title": "The Eternal Hunters",
    "roles": [
      "ADC"
    ]
  },
  {
    "id": 240,
    "name": "Kled",
    "key": "Kled",
    "title": "the Cantankerous Cavalier",
    "roles": [
      "TOP",
      "JUNGLE"
    ]
  },
  {
    "id": 96,
    "name": "Kog'Maw",
    "key": "KogMaw",
    "title": "the Mouth of the Abyss",
    "roles": [
      "ADC"
    ]
  },
  {
    "id": 7,
    "name": "LeBlanc",
    "key": "Leblanc",
    "title": "the Deceiver",
    "roles": [
      "MID",
      "JUNGLE"
    ]
  },
  {
    "id": 64,
    "name": "Lee Sin",
    "key": "LeeSin",
    "title": "the Blind Monk",
    "roles": [
      "TOP",
      "JUNGLE"
    ]
  },
  {
    "id": 89,
    "name": "Leona",
    "key": "Leona",
    "title": "the Radiant Dawn",
    "roles": [
      "TOP",
      "SUPPORT"
    ]
  },
  {
    "id": 876,
    "name": "Lillia",
    "key": "Lillia",
    "title": "the Bashful Bloom",
    "roles": [
      "TOP",
      "JUNGLE"
    ]
  },
  {
    "id": 127,
    "name": "Lissandra",
    "key": "Lissandra",
    "title": "the Ice Witch",
    "roles": [
      "MID",
      "SUPPORT"
    ]
  },
  {
    "id": 805,
    "name": "Locke",
    "key": "Locke",
    "title": "the Ashen Exorcist",
    "roles": [
      "MID",
      "JUNGLE"
    ]
  },
  {
    "id": 236,
    "name": "Lucian",
    "key": "Lucian",
    "title": "the Purifier",
    "roles": [
      "ADC"
    ]
  },
  {
    "id": 117,
    "name": "Lulu",
    "key": "Lulu",
    "title": "the Fae Sorceress",
    "roles": [
      "SUPPORT"
    ]
  },
  {
    "id": 99,
    "name": "Lux",
    "key": "Lux",
    "title": "the Lady of Luminosity",
    "roles": [
      "MID",
      "SUPPORT"
    ]
  },
  {
    "id": 54,
    "name": "Malphite",
    "key": "Malphite",
    "title": "Shard of the Monolith",
    "roles": [
      "TOP",
      "SUPPORT"
    ]
  },
  {
    "id": 90,
    "name": "Malzahar",
    "key": "Malzahar",
    "title": "the Prophet of the Void",
    "roles": [
      "MID",
      "SUPPORT"
    ]
  },
  {
    "id": 57,
    "name": "Maokai",
    "key": "Maokai",
    "title": "the Twisted Treant",
    "roles": [
      "TOP",
      "SUPPORT"
    ]
  },
  {
    "id": 11,
    "name": "Master Yi",
    "key": "MasterYi",
    "title": "the Wuju Bladesman",
    "roles": [
      "TOP",
      "JUNGLE"
    ]
  },
  {
    "id": 800,
    "name": "Mel",
    "key": "Mel",
    "title": "the Soul's Reflection",
    "roles": [
      "MID",
      "SUPPORT"
    ]
  },
  {
    "id": 902,
    "name": "Milio",
    "key": "Milio",
    "title": "The Gentle Flame",
    "roles": [
      "SUPPORT"
    ]
  },
  {
    "id": 21,
    "name": "Miss Fortune",
    "key": "MissFortune",
    "title": "the Bounty Hunter",
    "roles": [
      "ADC"
    ]
  },
  {
    "id": 82,
    "name": "Mordekaiser",
    "key": "Mordekaiser",
    "title": "the Iron Revenant",
    "roles": [
      "TOP",
      "JUNGLE"
    ]
  },
  {
    "id": 25,
    "name": "Morgana",
    "key": "Morgana",
    "title": "the Fallen",
    "roles": [
      "SUPPORT"
    ]
  },
  {
    "id": 950,
    "name": "Naafiri",
    "key": "Naafiri",
    "title": "the Hound of a Hundred Bites",
    "roles": [
      "MID",
      "JUNGLE"
    ]
  },
  {
    "id": 267,
    "name": "Nami",
    "key": "Nami",
    "title": "the Tidecaller",
    "roles": [
      "SUPPORT"
    ]
  },
  {
    "id": 75,
    "name": "Nasus",
    "key": "Nasus",
    "title": "the Curator of the Sands",
    "roles": [
      "TOP",
      "JUNGLE"
    ]
  },
  {
    "id": 111,
    "name": "Nautilus",
    "key": "Nautilus",
    "title": "the Titan of the Depths",
    "roles": [
      "TOP",
      "SUPPORT"
    ]
  },
  {
    "id": 518,
    "name": "Neeko",
    "key": "Neeko",
    "title": "the Curious Chameleon",
    "roles": [
      "MID",
      "SUPPORT"
    ]
  },
  {
    "id": 76,
    "name": "Nidalee",
    "key": "Nidalee",
    "title": "the Bestial Huntress",
    "roles": [
      "MID",
      "JUNGLE"
    ]
  },
  {
    "id": 895,
    "name": "Nilah",
    "key": "Nilah",
    "title": "the Joy Unbound",
    "roles": [
      "TOP",
      "JUNGLE"
    ]
  },
  {
    "id": 56,
    "name": "Nocturne",
    "key": "Nocturne",
    "title": "the Eternal Nightmare",
    "roles": [
      "TOP",
      "JUNGLE"
    ]
  },
  {
    "id": 20,
    "name": "Nunu & Willump",
    "key": "Nunu",
    "title": "the Boy and His Yeti",
    "roles": [
      "TOP",
      "SUPPORT"
    ]
  },
  {
    "id": 2,
    "name": "Olaf",
    "key": "Olaf",
    "title": "the Berserker",
    "roles": [
      "TOP",
      "JUNGLE"
    ]
  },
  {
    "id": 61,
    "name": "Orianna",
    "key": "Orianna",
    "title": "the Lady of Clockwork",
    "roles": [
      "MID",
      "SUPPORT"
    ]
  },
  {
    "id": 516,
    "name": "Ornn",
    "key": "Ornn",
    "title": "The Fire below the Mountain",
    "roles": [
      "TOP",
      "SUPPORT"
    ]
  },
  {
    "id": 80,
    "name": "Pantheon",
    "key": "Pantheon",
    "title": "the Unbreakable Spear",
    "roles": [
      "TOP",
      "JUNGLE"
    ]
  },
  {
    "id": 78,
    "name": "Poppy",
    "key": "Poppy",
    "title": "Keeper of the Hammer",
    "roles": [
      "TOP",
      "SUPPORT"
    ]
  },
  {
    "id": 555,
    "name": "Pyke",
    "key": "Pyke",
    "title": "the Bloodharbor Ripper",
    "roles": [
      "SUPPORT"
    ]
  },
  {
    "id": 246,
    "name": "Qiyana",
    "key": "Qiyana",
    "title": "Empress of the Elements",
    "roles": [
      "MID",
      "JUNGLE"
    ]
  },
  {
    "id": 133,
    "name": "Quinn",
    "key": "Quinn",
    "title": "Demacia's Wings",
    "roles": [
      "ADC"
    ]
  },
  {
    "id": 497,
    "name": "Rakan",
    "key": "Rakan",
    "title": "The Charmer",
    "roles": [
      "SUPPORT"
    ]
  },
  {
    "id": 33,
    "name": "Rammus",
    "key": "Rammus",
    "title": "the Armordillo",
    "roles": [
      "TOP",
      "SUPPORT"
    ]
  },
  {
    "id": 421,
    "name": "Rek'Sai",
    "key": "RekSai",
    "title": "the Void Burrower",
    "roles": [
      "TOP",
      "JUNGLE"
    ]
  },
  {
    "id": 526,
    "name": "Rell",
    "key": "Rell",
    "title": "the Iron Maiden",
    "roles": [
      "TOP",
      "SUPPORT"
    ]
  },
  {
    "id": 888,
    "name": "Renata Glasc",
    "key": "Renata",
    "title": "the Chem-Baroness",
    "roles": [
      "SUPPORT"
    ]
  },
  {
    "id": 58,
    "name": "Renekton",
    "key": "Renekton",
    "title": "the Butcher of the Sands",
    "roles": [
      "TOP",
      "JUNGLE"
    ]
  },
  {
    "id": 107,
    "name": "Rengar",
    "key": "Rengar",
    "title": "the Pridestalker",
    "roles": [
      "MID",
      "JUNGLE"
    ]
  },
  {
    "id": 92,
    "name": "Riven",
    "key": "Riven",
    "title": "the Exile",
    "roles": [
      "TOP",
      "JUNGLE"
    ]
  },
  {
    "id": 68,
    "name": "Rumble",
    "key": "Rumble",
    "title": "the Mechanized Menace",
    "roles": [
      "TOP",
      "JUNGLE"
    ]
  },
  {
    "id": 13,
    "name": "Ryze",
    "key": "Ryze",
    "title": "the Rune Mage",
    "roles": [
      "MID",
      "SUPPORT"
    ]
  },
  {
    "id": 360,
    "name": "Samira",
    "key": "Samira",
    "title": "the Desert Rose",
    "roles": [
      "ADC"
    ]
  },
  {
    "id": 113,
    "name": "Sejuani",
    "key": "Sejuani",
    "title": "Fury of the North",
    "roles": [
      "TOP",
      "SUPPORT"
    ]
  },
  {
    "id": 235,
    "name": "Senna",
    "key": "Senna",
    "title": "the Redeemer",
    "roles": [
      "ADC",
      "SUPPORT"
    ]
  },
  {
    "id": 147,
    "name": "Seraphine",
    "key": "Seraphine",
    "title": "the Starry-Eyed Songstress",
    "roles": [
      "SUPPORT"
    ]
  },
  {
    "id": 875,
    "name": "Sett",
    "key": "Sett",
    "title": "the Boss",
    "roles": [
      "TOP",
      "JUNGLE"
    ]
  },
  {
    "id": 35,
    "name": "Shaco",
    "key": "Shaco",
    "title": "the Demon Jester",
    "roles": [
      "MID",
      "JUNGLE"
    ]
  },
  {
    "id": 98,
    "name": "Shen",
    "key": "Shen",
    "title": "the Eye of Twilight",
    "roles": [
      "TOP",
      "SUPPORT"
    ]
  },
  {
    "id": 102,
    "name": "Shyvana",
    "key": "Shyvana",
    "title": "the Half-Dragon",
    "roles": [
      "TOP",
      "JUNGLE"
    ]
  },
  {
    "id": 27,
    "name": "Singed",
    "key": "Singed",
    "title": "the Mad Chemist",
    "roles": [
      "TOP",
      "SUPPORT"
    ]
  },
  {
    "id": 14,
    "name": "Sion",
    "key": "Sion",
    "title": "The Undead Juggernaut",
    "roles": [
      "TOP",
      "SUPPORT"
    ]
  },
  {
    "id": 15,
    "name": "Sivir",
    "key": "Sivir",
    "title": "the Battle Mistress",
    "roles": [
      "ADC"
    ]
  },
  {
    "id": 72,
    "name": "Skarner",
    "key": "Skarner",
    "title": "the Primordial Sovereign",
    "roles": [
      "TOP",
      "SUPPORT"
    ]
  },
  {
    "id": 901,
    "name": "Smolder",
    "key": "Smolder",
    "title": "the Fiery Fledgling",
    "roles": [
      "ADC"
    ]
  },
  {
    "id": 37,
    "name": "Sona",
    "key": "Sona",
    "title": "Maven of the Strings",
    "roles": [
      "SUPPORT"
    ]
  },
  {
    "id": 16,
    "name": "Soraka",
    "key": "Soraka",
    "title": "the Starchild",
    "roles": [
      "SUPPORT"
    ]
  },
  {
    "id": 50,
    "name": "Swain",
    "key": "Swain",
    "title": "the Noxian Grand General",
    "roles": [
      "MID",
      "SUPPORT"
    ]
  },
  {
    "id": 517,
    "name": "Sylas",
    "key": "Sylas",
    "title": "the Unshackled",
    "roles": [
      "MID",
      "SUPPORT"
    ]
  },
  {
    "id": 134,
    "name": "Syndra",
    "key": "Syndra",
    "title": "the Dark Sovereign",
    "roles": [
      "MID",
      "SUPPORT"
    ]
  },
  {
    "id": 223,
    "name": "Tahm Kench",
    "key": "TahmKench",
    "title": "The River King",
    "roles": [
      "TOP",
      "SUPPORT"
    ]
  },
  {
    "id": 163,
    "name": "Taliyah",
    "key": "Taliyah",
    "title": "the Stoneweaver",
    "roles": [
      "MID",
      "SUPPORT"
    ]
  },
  {
    "id": 91,
    "name": "Talon",
    "key": "Talon",
    "title": "the Blade's Shadow",
    "roles": [
      "MID",
      "JUNGLE"
    ]
  },
  {
    "id": 44,
    "name": "Taric",
    "key": "Taric",
    "title": "the Shield of Valoran",
    "roles": [
      "SUPPORT"
    ]
  },
  {
    "id": 17,
    "name": "Teemo",
    "key": "Teemo",
    "title": "the Swift Scout",
    "roles": [
      "ADC"
    ]
  },
  {
    "id": 412,
    "name": "Thresh",
    "key": "Thresh",
    "title": "the Chain Warden",
    "roles": [
      "SUPPORT"
    ]
  },
  {
    "id": 18,
    "name": "Tristana",
    "key": "Tristana",
    "title": "the Yordle Gunner",
    "roles": [
      "ADC"
    ]
  },
  {
    "id": 48,
    "name": "Trundle",
    "key": "Trundle",
    "title": "the Troll King",
    "roles": [
      "TOP",
      "JUNGLE"
    ]
  },
  {
    "id": 23,
    "name": "Tryndamere",
    "key": "Tryndamere",
    "title": "the Barbarian King",
    "roles": [
      "TOP",
      "JUNGLE"
    ]
  },
  {
    "id": 4,
    "name": "Twisted Fate",
    "key": "TwistedFate",
    "title": "the Card Master",
    "roles": [
      "ADC",
      "MID",
      "SUPPORT"
    ]
  },
  {
    "id": 29,
    "name": "Twitch",
    "key": "Twitch",
    "title": "the Plague Rat",
    "roles": [
      "ADC"
    ]
  },
  {
    "id": 77,
    "name": "Udyr",
    "key": "Udyr",
    "title": "the Spirit Walker",
    "roles": [
      "TOP",
      "JUNGLE"
    ]
  },
  {
    "id": 6,
    "name": "Urgot",
    "key": "Urgot",
    "title": "the Dreadnought",
    "roles": [
      "TOP",
      "JUNGLE"
    ]
  },
  {
    "id": 110,
    "name": "Varus",
    "key": "Varus",
    "title": "the Arrow of Retribution",
    "roles": [
      "ADC"
    ]
  },
  {
    "id": 67,
    "name": "Vayne",
    "key": "Vayne",
    "title": "the Night Hunter",
    "roles": [
      "ADC"
    ]
  },
  {
    "id": 45,
    "name": "Veigar",
    "key": "Veigar",
    "title": "the Tiny Master of Evil",
    "roles": [
      "MID",
      "SUPPORT"
    ]
  },
  {
    "id": 161,
    "name": "Vel'Koz",
    "key": "Velkoz",
    "title": "the Eye of the Void",
    "roles": [
      "MID",
      "SUPPORT"
    ]
  },
  {
    "id": 711,
    "name": "Vex",
    "key": "Vex",
    "title": "the Gloomist",
    "roles": [
      "MID",
      "SUPPORT"
    ]
  },
  {
    "id": 254,
    "name": "Vi",
    "key": "Vi",
    "title": "the Piltover Enforcer",
    "roles": [
      "TOP",
      "JUNGLE"
    ]
  },
  {
    "id": 234,
    "name": "Viego",
    "key": "Viego",
    "title": "The Ruined King",
    "roles": [
      "TOP",
      "JUNGLE"
    ]
  },
  {
    "id": 112,
    "name": "Viktor",
    "key": "Viktor",
    "title": "the Herald of the Arcane",
    "roles": [
      "MID",
      "SUPPORT"
    ]
  },
  {
    "id": 8,
    "name": "Vladimir",
    "key": "Vladimir",
    "title": "the Crimson Reaper",
    "roles": [
      "MID",
      "SUPPORT"
    ]
  },
  {
    "id": 106,
    "name": "Volibear",
    "key": "Volibear",
    "title": "the Relentless Storm",
    "roles": [
      "TOP",
      "JUNGLE"
    ]
  },
  {
    "id": 19,
    "name": "Warwick",
    "key": "Warwick",
    "title": "the Uncaged Wrath of Zaun",
    "roles": [
      "TOP",
      "JUNGLE"
    ]
  },
  {
    "id": 62,
    "name": "Wukong",
    "key": "MonkeyKing",
    "title": "the Monkey King",
    "roles": [
      "TOP",
      "JUNGLE"
    ]
  },
  {
    "id": 498,
    "name": "Xayah",
    "key": "Xayah",
    "title": "the Rebel",
    "roles": [
      "ADC"
    ]
  },
  {
    "id": 101,
    "name": "Xerath",
    "key": "Xerath",
    "title": "the Magus Ascendant",
    "roles": [
      "MID",
      "SUPPORT"
    ]
  },
  {
    "id": 5,
    "name": "Xin Zhao",
    "key": "XinZhao",
    "title": "the Seneschal of Demacia",
    "roles": [
      "TOP",
      "JUNGLE"
    ]
  },
  {
    "id": 157,
    "name": "Yasuo",
    "key": "Yasuo",
    "title": "the Unforgiven",
    "roles": [
      "TOP",
      "JUNGLE"
    ]
  },
  {
    "id": 777,
    "name": "Yone",
    "key": "Yone",
    "title": "the Unforgotten",
    "roles": [
      "TOP",
      "JUNGLE"
    ]
  },
  {
    "id": 83,
    "name": "Yorick",
    "key": "Yorick",
    "title": "Shepherd of Souls",
    "roles": [
      "TOP",
      "JUNGLE"
    ]
  },
  {
    "id": 804,
    "name": "Yunara",
    "key": "Yunara",
    "title": "the Unbroken Faith",
    "roles": [
      "ADC"
    ]
  },
  {
    "id": 350,
    "name": "Yuumi",
    "key": "Yuumi",
    "title": "the Magical Cat",
    "roles": [
      "SUPPORT"
    ]
  },
  {
    "id": 904,
    "name": "Zaahen",
    "key": "Zaahen",
    "title": "The Unsundered",
    "roles": [
      "TOP",
      "JUNGLE"
    ]
  },
  {
    "id": 154,
    "name": "Zac",
    "key": "Zac",
    "title": "the Secret Weapon",
    "roles": [
      "TOP",
      "SUPPORT"
    ]
  },
  {
    "id": 238,
    "name": "Zed",
    "key": "Zed",
    "title": "the Master of Shadows",
    "roles": [
      "MID",
      "JUNGLE"
    ]
  },
  {
    "id": 221,
    "name": "Zeri",
    "key": "Zeri",
    "title": "The Spark of Zaun",
    "roles": [
      "ADC"
    ]
  },
  {
    "id": 115,
    "name": "Ziggs",
    "key": "Ziggs",
    "title": "the Hexplosives Expert",
    "roles": [
      "MID",
      "SUPPORT"
    ]
  },
  {
    "id": 26,
    "name": "Zilean",
    "key": "Zilean",
    "title": "the Chronokeeper",
    "roles": [
      "SUPPORT"
    ]
  },
  {
    "id": 142,
    "name": "Zoe",
    "key": "Zoe",
    "title": "the Aspect of Twilight",
    "roles": [
      "MID",
      "SUPPORT"
    ]
  },
  {
    "id": 143,
    "name": "Zyra",
    "key": "Zyra",
    "title": "Rise of the Thorns",
    "roles": [
      "MID",
      "SUPPORT"
    ]
  }
];

export const CHAMPION_BUILDS: Record<number, ChampionBuild> = {
  // Illaoi (420)
  420: {
    championId: 420,
    championName: 'Illaoi',
    key: 'Illaoi',
    role: 'TOP',
    tier: 'S',
    winRate: 52.4,
    pickRate: 4.8,
    banRate: 6.2,
    runes: {
      primaryStyleId: 8400,
      primaryStyleName: 'Resolve',
      subStyleId: 8000,
      subStyleName: 'Precision',
      selectedPerkIds: [8437, 8446, 8473, 8451, 8009, 8299, 5008, 5008, 5001],
      keystoneId: 8437,
      keystoneName: 'Grasp of the Undying',
      keystoneIcon: 'https://ddragon.leagueoflegends.com/cdn/img/perk-images/Styles/Resolve/GraspOfTheUndying/GraspOfTheUndying.png'
    },
    secondaryRunes: {
      primaryStyleId: 8000,
      primaryStyleName: 'Precision',
      subStyleId: 8400,
      subStyleName: 'Resolve',
      selectedPerkIds: [8010, 8009, 9105, 8299, 8446, 8473, 5008, 5008, 5001],
      keystoneId: 8010,
      keystoneName: 'Conqueror',
      keystoneIcon: 'https://ddragon.leagueoflegends.com/cdn/img/perk-images/Styles/Precision/Conqueror/Conqueror.png'
    },
    spells: [4, 12], // Flash, TP
    skillOrder: ['E', 'Q', 'W', 'E', 'E', 'R', 'E', 'Q', 'E', 'Q', 'R', 'Q', 'Q', 'W', 'W', 'R', 'W', 'W'],
    skillPriority: 'E > Q > W',
    startingItems: [
      { id: 1054, name: "Doran's Shield" },
      { id: 2003, name: 'Health Potion' }
    ],
    coreItems: [
      { id: 3071, name: 'Black Cleaver' },
      { id: 6631, name: 'Stridebreaker' },
      { id: 3053, name: "Sterak's Gage" }
    ],
    situationalItems: [
      { id: 3068, name: 'Sunfire Aegis' },
      { id: 4401, name: 'Force of Nature' },
      { id: 3075, name: 'Thornmail' }
    ],
    boots: { id: 3047, name: 'Plated Steelcaps' },
    counters: {
      strongAgainst: [
        { name: 'Sion', winRate: 56.8 },
        { name: 'Tahm Kench', winRate: 55.4 },
        { name: 'Nasus', winRate: 54.9 }
      ],
      weakAgainst: [
        { name: 'Kayle', winRate: 44.5 },
        { name: 'Vayne', winRate: 45.2 },
        { name: 'Yorick', winRate: 46.1 }
      ]
    },
    tips: [
      'Land Test of Spirit (E) before casting Leap of Faith (R) for an extra tentacle spawn.',
      'Fight near walls where your tentacles can slam multiple times.',
      'Never trade without at least 1-2 active tentacles nearby.'
    ]
  },

  // Aatrox (266)
  266: {
    championId: 266,
    championName: 'Aatrox',
    key: 'Aatrox',
    role: 'TOP',
    tier: 'S+',
    winRate: 51.2,
    pickRate: 11.4,
    banRate: 8.5,
    runes: {
      primaryStyleId: 8000,
      primaryStyleName: 'Precision',
      subStyleId: 8400,
      subStyleName: 'Resolve',
      selectedPerkIds: [8010, 9111, 9105, 8299, 8444, 8451, 5008, 5008, 5001],
      keystoneId: 8010,
      keystoneName: 'Conqueror',
      keystoneIcon: 'https://ddragon.leagueoflegends.com/cdn/img/perk-images/Styles/Precision/Conqueror/Conqueror.png'
    },
    secondaryRunes: {
      primaryStyleId: 8100,
      primaryStyleName: 'Domination',
      subStyleId: 8000,
      subStyleName: 'Precision',
      selectedPerkIds: [8128, 8143, 8138, 8135, 9111, 8299, 5008, 5008, 5001],
      keystoneId: 8128,
      keystoneName: 'Dark Harvest',
      keystoneIcon: 'https://ddragon.leagueoflegends.com/cdn/img/perk-images/Styles/Domination/DarkHarvest/DarkHarvest.png'
    },
    spells: [4, 12],
    skillOrder: ['Q', 'E', 'W', 'Q', 'Q', 'R', 'Q', 'E', 'Q', 'E', 'R', 'E', 'E', 'W', 'W', 'R', 'W', 'W'],
    skillPriority: 'Q > E > W',
    startingItems: [
      { id: 1055, name: "Doran's Blade" },
      { id: 2003, name: 'Health Potion' }
    ],
    coreItems: [
      { id: 6692, name: 'Eclipse' },
      { id: 6694, name: 'Sundered Sky' },
      { id: 3071, name: 'Black Cleaver' }
    ],
    situationalItems: [
      { id: 3053, name: "Sterak's Gage" },
      { id: 6695, name: "Serpents Fang" },
      { id: 3026, name: 'Guardian Angel' }
    ],
    boots: { id: 3047, name: 'Plated Steelcaps' },
    counters: {
      strongAgainst: [
        { name: 'Sion', winRate: 54.3 },
        { name: 'Gangplank', winRate: 53.8 },
        { name: 'Kennen', winRate: 53.2 }
      ],
      weakAgainst: [
        { name: 'Irelia', winRate: 46.2 },
        { name: 'Fiora', winRate: 46.7 },
        { name: 'Kled', winRate: 47.1 }
      ]
    },
    tips: [
      'Use E (Umbral Dash) during Q animation to adjust the sweet spot landing position.',
      'World Ender (R) resets duration on champion takedowns and amplifies all self-healing.'
    ]
  },

  // Jinx (222)
  222: {
    championId: 222,
    championName: 'Jinx',
    key: 'Jinx',
    role: 'ADC',
    tier: 'S+',
    winRate: 52.8,
    pickRate: 19.3,
    banRate: 7.1,
    runes: {
      primaryStyleId: 8000,
      primaryStyleName: 'Precision',
      subStyleId: 8200,
      subStyleName: 'Sorcery',
      selectedPerkIds: [8004, 9101, 9104, 8014, 8210, 8236, 5005, 5008, 5001],
      keystoneId: 8004,
      keystoneName: 'Lethal Tempo',
      keystoneIcon: 'https://ddragon.leagueoflegends.com/cdn/img/perk-images/Styles/Precision/LethalTempo/LethalTempoTemp.png'
    },
    secondaryRunes: {
      primaryStyleId: 8000,
      primaryStyleName: 'Precision',
      subStyleId: 8300,
      subStyleName: 'Inspiration',
      selectedPerkIds: [8008, 9101, 9104, 8017, 8304, 8345, 5005, 5008, 5001],
      keystoneId: 8008,
      keystoneName: 'Fleet Footwork',
      keystoneIcon: 'https://ddragon.leagueoflegends.com/cdn/img/perk-images/Styles/Precision/FleetFootwork/FleetFootwork.png'
    },
    spells: [4, 7], // Flash, Heal
    skillOrder: ['Q', 'W', 'E', 'Q', 'Q', 'R', 'Q', 'W', 'Q', 'W', 'R', 'W', 'W', 'E', 'E', 'R', 'E', 'E'],
    skillPriority: 'Q > W > E',
    startingItems: [
      { id: 1055, name: "Doran's Blade" },
      { id: 2003, name: 'Health Potion' }
    ],
    coreItems: [
      { id: 3031, name: 'Infinity Edge' },
      { id: 3094, name: 'Rapid Firecannon' },
      { id: 3085, name: "Runaan's Hurricane" }
    ],
    situationalItems: [
      { id: 3036, name: "Lord Dominik's Regards" },
      { id: 3072, name: 'Bloodthirster' },
      { id: 3026, name: 'Guardian Angel' }
    ],
    boots: { id: 3006, name: "Berserker's Greaves" },
    counters: {
      strongAgainst: [
        { name: 'Aphelios', winRate: 54.1 },
        { name: 'Zeri', winRate: 53.6 },
        { name: 'KaiSa', winRate: 52.9 }
      ],
      weakAgainst: [
        { name: 'Twitch', winRate: 46.4 },
        { name: 'Draven', winRate: 47.0 },
        { name: 'Samira', winRate: 47.8 }
      ]
    },
    tips: [
      'Switch to Fishbones (Rockets) for long range poke and teamfight AoE, use Pow-Pow (Minigun) for towers and single-target DPS.',
      'Place Flame Chompers (E) behind enemy champions or in choke points to self-peel.'
    ]
  },

  // Ahri (103)
  103: {
    championId: 103,
    championName: 'Ahri',
    key: 'Ahri',
    role: 'MID',
    tier: 'S',
    winRate: 51.5,
    pickRate: 10.2,
    banRate: 4.5,
    runes: {
      primaryStyleId: 8100,
      primaryStyleName: 'Domination',
      subStyleId: 8200,
      subStyleName: 'Sorcery',
      selectedPerkIds: [8112, 8139, 8138, 8106, 8226, 8210, 5008, 5008, 5001],
      keystoneId: 8112,
      keystoneName: 'Electrocute',
      keystoneIcon: 'https://ddragon.leagueoflegends.com/cdn/img/perk-images/Styles/Domination/Electrocute/Electrocute.png'
    },
    secondaryRunes: {
      primaryStyleId: 8300,
      primaryStyleName: 'Inspiration',
      subStyleId: 8100,
      subStyleName: 'Domination',
      selectedPerkIds: [8351, 8304, 8345, 8347, 8139, 8106, 5008, 5008, 5001],
      keystoneId: 8351,
      keystoneName: 'Glacial Augment',
      keystoneIcon: 'https://ddragon.leagueoflegends.com/cdn/img/perk-images/Styles/Inspiration/GlacialAugment/GlacialAugment.png'
    },
    spells: [4, 12],
    skillOrder: ['Q', 'W', 'E', 'Q', 'Q', 'R', 'Q', 'W', 'Q', 'W', 'R', 'W', 'W', 'E', 'E', 'R', 'E', 'E'],
    skillPriority: 'Q > W > E',
    startingItems: [
      { id: 1056, name: "Doran's Ring" },
      { id: 2003, name: 'Health Potion' }
    ],
    coreItems: [
      { id: 6653, name: "Liandry's Torment" },
      { id: 4645, name: 'Shadowflame' },
      { id: 3089, name: "Rabadon's Deathcap" }
    ],
    situationalItems: [
      { id: 3157, name: "Zhonya's Hourglass" },
      { id: 3135, name: 'Void Staff' },
      { id: 3102, name: "Banshee's Veil" }
    ],
    boots: { id: 3020, name: "Sorcerer's Shoes" },
    counters: {
      strongAgainst: [
        { name: 'Kassadin', winRate: 53.7 },
        { name: 'Azir', winRate: 53.1 },
        { name: 'Orianna', winRate: 52.4 }
      ],
      weakAgainst: [
        { name: 'Tristana', winRate: 46.8 },
        { name: 'Akshan', winRate: 47.3 },
        { name: 'Naafiri', winRate: 47.9 }
      ]
    },
    tips: [
      'Charm (E) cancels dashes and amplifies follow-up magic damage burst.',
      'Spirit Rush (R) grants up to 3 dashes and gains an additional recast on takedowns.'
    ]
  },

  // Lee Sin (64)
  64: {
    championId: 64,
    championName: 'Lee Sin',
    key: 'LeeSin',
    role: 'JUNGLE',
    tier: 'S',
    winRate: 49.8,
    pickRate: 15.6,
    banRate: 9.3,
    runes: {
      primaryStyleId: 8000,
      primaryStyleName: 'Precision',
      subStyleId: 8300,
      subStyleName: 'Inspiration',
      selectedPerkIds: [8010, 9111, 9104, 8014, 8304, 8347, 5008, 5008, 5001],
      keystoneId: 8010,
      keystoneName: 'Conqueror',
      keystoneIcon: 'https://ddragon.leagueoflegends.com/cdn/img/perk-images/Styles/Precision/Conqueror/Conqueror.png'
    },
    secondaryRunes: {
      primaryStyleId: 8100,
      primaryStyleName: 'Domination',
      subStyleId: 8000,
      subStyleName: 'Precision',
      selectedPerkIds: [8112, 8143, 8138, 8105, 9111, 8014, 5008, 5008, 5001],
      keystoneId: 8112,
      keystoneName: 'Electrocute',
      keystoneIcon: 'https://ddragon.leagueoflegends.com/cdn/img/perk-images/Styles/Domination/Electrocute/Electrocute.png'
    },
    spells: [4, 11], // Flash, Smite
    skillOrder: ['Q', 'W', 'E', 'Q', 'Q', 'R', 'Q', 'W', 'Q', 'W', 'R', 'W', 'W', 'E', 'E', 'R', 'E', 'E'],
    skillPriority: 'Q > W > E',
    startingItems: [
      { id: 1102, name: 'Gustwalker Hatchling' },
      { id: 2003, name: 'Health Potion' }
    ],
    coreItems: [
      { id: 6692, name: 'Eclipse' },
      { id: 6694, name: 'Sundered Sky' },
      { id: 3071, name: 'Black Cleaver' }
    ],
    situationalItems: [
      { id: 3053, name: "Sterak's Gage" },
      { id: 3026, name: 'Guardian Angel' },
      { id: 6333, name: "Death's Dance" }
    ],
    boots: { id: 3047, name: 'Plated Steelcaps' },
    counters: {
      strongAgainst: [
        { name: 'Nidalee', winRate: 53.4 },
        { name: 'Karthus', winRate: 52.9 },
        { name: 'Graves', winRate: 52.0 }
      ],
      weakAgainst: [
        { name: 'Warwick', winRate: 46.1 },
        { name: 'Poppy', winRate: 46.8 },
        { name: 'Udyr', winRate: 47.3 }
      ]
    },
    tips: [
      'Insec Kick combo: Q1 -> Q2 -> Ward Hop (W) behind target -> Dragon Rage (R) towards your team.',
      'Always weave basic attacks between spell casts to utilize Flurry energy restoration.'
    ]
  },

  // Thresh (412)
  412: {
    championId: 412,
    championName: 'Thresh',
    key: 'Thresh',
    role: 'SUPPORT',
    tier: 'S+',
    winRate: 51.6,
    pickRate: 14.2,
    banRate: 5.7,
    runes: {
      primaryStyleId: 8300,
      primaryStyleName: 'Inspiration',
      subStyleId: 8400,
      subStyleName: 'Resolve',
      selectedPerkIds: [8351, 8306, 8345, 8347, 8473, 8453, 5007, 5008, 5001],
      keystoneId: 8351,
      keystoneName: 'Glacial Augment',
      keystoneIcon: 'https://ddragon.leagueoflegends.com/cdn/img/perk-images/Styles/Inspiration/GlacialAugment/GlacialAugment.png'
    },
    secondaryRunes: {
      primaryStyleId: 8400,
      primaryStyleName: 'Resolve',
      subStyleId: 8300,
      subStyleName: 'Inspiration',
      selectedPerkIds: [8439, 8463, 8473, 8453, 8345, 8347, 5007, 5008, 5001],
      keystoneId: 8439,
      keystoneName: 'Aftershock',
      keystoneIcon: 'https://ddragon.leagueoflegends.com/cdn/img/perk-images/Styles/Resolve/VeteranAftershock/VeteranAftershock.png'
    },
    spells: [4, 14], // Flash, Ignite
    skillOrder: ['Q', 'E', 'W', 'Q', 'Q', 'R', 'Q', 'W', 'Q', 'W', 'R', 'W', 'W', 'E', 'E', 'R', 'E', 'E'],
    skillPriority: 'Q > W > E',
    startingItems: [
      { id: 3865, name: 'World Atlas' },
      { id: 2003, name: 'Health Potion' }
    ],
    coreItems: [
      { id: 3190, name: 'Locket of the Iron Solari' },
      { id: 3050, name: "Zeke's Convergence" },
      { id: 3109, name: "Knight's Vow" }
    ],
    situationalItems: [
      { id: 3107, name: 'Redemption' },
      { id: 3001, name: 'Abyssal Mask' },
      { id: 3075, name: 'Thornmail' }
    ],
    boots: { id: 3117, name: 'Mobility Boots' },
    counters: {
      strongAgainst: [
        { name: 'Yuumi', winRate: 54.8 },
        { name: 'Leona', winRate: 53.2 },
        { name: 'Nautilus', winRate: 52.5 }
      ],
      weakAgainst: [
        { name: 'Morgana', winRate: 46.3 },
        { name: 'Zyra', winRate: 47.1 },
        { name: 'Brand', winRate: 47.6 }
      ]
    },
    tips: [
      'Flay (E) before Death Sentence (Q) for a guaranteed hook against mobile targets.',
      'Lantern (W) collects souls automatically from distance and can save overextended allies.'
    ]
  }
};

// Dynamic build generator for all 169+ champions
export function getChampionBuild(championId: number, championName?: string): ChampionBuild {
  if (CHAMPION_BUILDS[championId]) return CHAMPION_BUILDS[championId];
  const list = patchService.getChampions() || CHAMPIONS_LIST;
  const champ = list.find((ch) => ch.id === championId) || {
    id: championId,
    name: championName || 'Champion',
    key: championName || 'Champion',
    roles: ['MID']
  };
  return buildChampionProfile(champ.id, champ.name, champ.key);
}

export function getChampionIcon(championIdOrKey: number | string): string {
  if (typeof championIdOrKey === 'number') {
    const list = patchService.getChampions() || CHAMPIONS_LIST;
    const found = list.find((c) => c.id === championIdOrKey);
    if (found) {
      return patchService.getChampionIconUrl(found.key);
    }
    return patchService.getChampionIconUrl(String(championIdOrKey));
  }
  return patchService.getChampionIconUrl(championIdOrKey);
}

export function getChampionSplash(championKey: string): string {
  return patchService.getChampionSplashUrl(championKey);
}
