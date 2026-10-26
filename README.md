# ⚔️ RiftCore Companion 2.0

<div align="center">

![License](https://img.shields.io/badge/license-MIT-blue.svg)
![Hybrid Architecture](https://img.shields.io/badge/engine-Rust_+_DirectComposition_Hybrid-9cf.svg?logo=rust)
![React](https://img.shields.io/badge/frontend-React_19_+_TypeScript-61dafb.svg?logo=react)
![RAM Footprint](https://img.shields.io/badge/idle_memory-~40_MB_RAM-brightgreen.svg)
![Overlay](https://img.shields.io/badge/overlay-100%25_DirectComposition_Transparent-00c8ff.svg)
![Ad-Free](https://img.shields.io/badge/ads-ZERO_ADS-crimson.svg)

**The ultra-lightweight, ad-free, high-performance League of Legends companion & zero-whitebox overlay.**

[Features](#-features) • [Performance Comparison](#-performance-benchmark) • [Hybrid Architecture](#-hybrid-architecture) • [Installation](#-installation) • [Build from Source](#-building-from-source) • [License](#-license)

</div>

---

## ⚡ Why RiftCore 2.0?

Tired of bloated League apps using **1GB+ RAM**, showing intrusive ads, and dropping your FPS?

- 🚀 **~40 MB RAM:** 25x lighter than Overwolf. Boots in 0.1s.
- 🎯 **Clean In-Game Overlay:** 100% transparent, click-through, and zero frame drops.
- 🧹 **Auto-RAM Reclaim:** Overlay runs only during matches and closes automatically after.
- 🔮 **1-Click Runes & Spells:** Meta builds imported directly into your League client.
- ⚡ **Instant Auto-Accept:** Never miss a queue pop.
- 🛡️ **Zero Ads & Vanguard Safe:** 100% free, open-source, and fully compliant with Riot policies.

---

## 🚀 Features

### ⚡ Instant Auto-Accept
Queue pops are detected and accepted instantly with audio alerts.

### 🔮 1-Click Runes & Spells
High-winrate meta rune pages and summoner spells imported directly into your client for all 160+ champions.

### 🎯 In-Game HUD & Minimap Timers
- **Minimap Timers:** Dragon, Baron, Voidgrubs, and Buffs cleanly docked above the minimap.
- **Scoreboard Gold Lead:** Real-time team gold differential at the top center.
- **CS Diff Tracker:** CS score and CS/min pill beside your game clock.
- **Level-Up Indicator:** Shows recommended ability upgrades in real time.
- **Hotkeys:** <kbd>Alt + ~</kbd> or <kbd>Ctrl + Space</kbd> to toggle visibility.

### 📊 Ranked History & Stats
Live rank, LP, win rates, and recent match stats with MVP/ACE badges.

---

## 📊 Performance Comparison

| Metric | Other Apps (Overwolf / Porofessor) | ⚡ **RiftCore 2.0** |
| :--- | :---: | :---: |
| **Idle Companion RAM** | ~800 MB – 1.4 GB | **~40 MB** |
| **In-Game Overlay RAM** | ~1.2 GB+ persistent | **~300 MB** *(freed on game end)* |
| **Overlay Transparency** | Washed out / borders | **100% Clean & Click-Through** |
| **Startup Time** | 4.5 – 9.0s | **< 0.1s** |
| **Ads & Trackers** | Heavy (video/banners) | **ZERO** |

---

## 📦 Downloads & Installation

### Direct Downloads (Latest Release v2.0.0)
| Package | Type | Download Link | Description |
| :--- | :--- | :--- | :--- |
| **Windows Installer** | Standalone Setup | [📥 **Download RiftCore-Setup-2.0.0.exe**](https://github.com/kotsioskl2/riftcore-companion/releases/latest) | Complete 1-Click Installer (Start Menu & Desktop shortcuts) |
| **Portable ZIP Archive** | Portable ZIP | [📦 **Download RiftCore-Portable-2.0.0.zip**](https://github.com/kotsioskl2/riftcore-companion/releases/latest) | Zero-install, unzips to any folder |
| **Portable Directory** | Local Extracted | [📁 **Browse RiftCore_Rust_Portable/**](./RiftCore_Rust_Portable/) | Run `RiftCore.exe` directly |

### Option A: Windows Installer (Recommended)
1. Download [**RiftCore-Setup-2.0.0.exe**](https://github.com/kotsioskl2/riftcore-companion/releases/latest).
2. Run the setup and choose your installation directory.
3. Desktop and Start Menu shortcuts with the custom RiftCore emblem will be created automatically.

### Option B: Standalone Portable `.exe`
1. Download [**RiftCore-Portable-2.0.0.zip**](https://github.com/kotsioskl2/riftcore-companion/releases/latest) or open [`RiftCore_Rust_Portable/`](./RiftCore_Rust_Portable/).
2. Double-click `RiftCore.exe` to launch instantly with zero installation required.

---

## 🛠️ Building from Source

### Prerequisites
- [Node.js](https://nodejs.org/) (v18+) & `npm`
- [Rust](https://rustup.rs/) (v1.80+ with `x86_64-pc-windows-gnu` or `msvc` toolchain)
- MinGW / GCC (e.g. `w64devkit` or MSVC C++ Build Tools)
- [NSIS](https://nsis.sourceforge.io/) (for compiling the installer)

### 1-Click Master Build
Simply run the master build batch script from the repository root:
```cmd
build_all.bat
```
This automated script will:
1. Package the React 19 frontend and DirectComposition overlay engine into `app.asar`.
2. Compile the high-performance native Rust companion (`riftcore-native.exe`) in release profile.
3. Assemble the portable distribution folder `RiftCore_Rust_Portable/`.
4. Compile the Windows NSIS Setup installer in `installer_output/RiftCore-Setup-2.0.0.exe`.
5. Generate the portable ZIP package in `installer_output/RiftCore-Portable-2.0.0.zip`.

---

## 🏗️ Hybrid Architecture

```
riftcore-companion/
├── src/                          # Modern React 19 + TypeScript Frontend
│   ├── components/               # Minimalist Red & Blue UI Views
│   │   ├── TitleBar.tsx          # Window controls, live client status pill
│   │   ├── ChampSelectView.tsx   # Meta builds, 1-click rune & spell import
│   │   ├── LiveGameOverlay.tsx   # Objective & enemy cooldown HUD
│   │   ├── RiftFloatingOverlay.tsx # Minimap dock & scoreboard diff HUD
│   │   ├── SummonerProfileView.tsx # Ranked LP stats & match history
│   │   └── ChampionTierListView.tsx# Champion tier list & counter picks
│   ├── rustBridge.ts             # Bi-directional IPC bridge
│   └── index.css                 # Obsidian styling with Electric Cyan & Crimson
├── riftcore-native/              # Native Rust Companion Core (~40 MB)
│   ├── src/main.rs               # Tokio async engine, LCU client, process watcher,
│   │                             # and dynamic overlay lifecycle manager
│   └── Cargo.toml                # Dependencies: wry, tao, tokio, reqwest
├── electron/                     # DirectComposition Standalone Overlay Engine
│   ├── main.js                   # 100% transparent click-through HUD process
│   └── preload.js                # Secure bridge to Live Client Data port 2999
├── scripts/
│   ├── build_electron_exe.mjs    # High-speed ASAR overlay packager
│   ├── bump_version.mjs          # Automatic semver synchronizer
│   └── publish_release.ps1       # GitHub Releases API asset publisher
├── installer/                    # NSIS Windows Installer Scripts
│   └── installer.nsi             # Modern UI 2 solid LZMA installer definition
├── build_all.bat                 # 1-Click end-to-end master builder
└── package.json
```

---

## ☕ Support & Fuel Development

> *"Building and hosting apps takes time and caffeine. If you want to help fuel the next project:"*

<div align="center">

[![ko-fi](https://ko-fi.com/img/githubbutton_sm.svg)](https://ko-fi.com/X6I629C814)

<br/>

<a href="https://ko-fi.com/X6I629C814">
  <img src="./kofi_qr.png" alt="Scan with Camera to Tip on Ko-fi" width="130" style="border-radius: 10px; box-shadow: 0 0 15px rgba(0, 200, 255, 0.3);" />
</a>

<br/>

*Scan with your phone camera or click above to pay with **Apple Pay**, **Google Pay**, or **Card** (Zero account required!)*

</div>

### ⚡ Direct Links:
| ☕ [**Support on Ko-fi (Apple Pay / G-Pay / Card)**](https://ko-fi.com/X6I629C814) | ⚡ [**Tip via Revolut**](https://revolut.me/kkleovoulou) |
| :---: | :---: |

---

## 🤝 Contributing

Contributions, issues, and feature requests are welcome!
Feel free to check the [issues page](https://github.com/kotsioskl2/riftcore-companion/issues).

1. Fork the Project
2. Create your Feature Branch (`git checkout -b feature/AmazingFeature`)
3. Commit your Changes (`git commit -m 'Add some AmazingFeature'`)
4. Push to the Branch (`git push origin feature/AmazingFeature`)
5. Open a Pull Request

---

## 📜 License & Riot Games Legal Notice

Distributed under the **MIT License**. See [`LICENSE`](LICENSE) for more information.

> **⚖️ Riot Games "Legal Jibber Jabber":**
> RiftCore Companion isn't endorsed by Riot Games and doesn't reflect the views or opinions of Riot Games or anyone officially involved in producing or managing Riot Games properties. Riot Games, and all associated properties are trademarks or registered trademarks of Riot Games, Inc.

---

<div align="center">
Made with ❤️ for League of Legends players who want performance without compromise.
</div>
