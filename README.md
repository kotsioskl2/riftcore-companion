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

Tired of bloated League companion apps (Overwolf, Porofessor, commercial ad-heavy apps) consuming **1GB+ of RAM**, injecting intrusive audio/video ads, and causing frame drops mid-game?

**RiftCore 2.0** introduces the **Hybrid Architecture**:
- **Native Rust Companion:** Idles around **~40 MB RAM** (25x less RAM than competitors), booting in **0.1s** with instant Auto-Accept, 1-Click Runes, and AI Draft Coaching.
- **Zero White-Box DirectComposition Overlay:** Eliminates the classic Windows child HWND white rectangle bug entirely using a dedicated hardware-accelerated DirectComposition overlay with 100% per-pixel transparency and true click-through.
- **Dynamic Memory Reclaiming:** Spawns the overlay only while in a match and fully terminates it when the match ends, dropping RAM right back down to ~40 MB.
- **Zero Ads, Zero Trackers:** Pure utility, 100% free and open-source forever.
- **Vanguard Safe:** Interacts only through Riot's official LCU REST & WebSocket endpoints and local game memory port 2999.

---

## 🚀 Features

### 1. ⚡ Instant Auto-Accept
- Automatically detects match queue pops and accepts instantly.
- Plays crisp audio cues and alerts.

### 2. 🔮 1-Click Rune & Summoner Spell Import
- High-winrate meta builds for all 160+ champions across Top, Jungle, Mid, ADC, and Support.
- Imports complete rune trees (Primary, Secondary, Stat Shards) directly into your active League client perk page with one click.
- Auto-assigns recommended Flash + Summoner Spell combinations.

### 3. 🎯 True In-Game HUD & Minimap Objectives
- Real-time in-game overlay querying local port 2999 Live Client Data.
- **Minimap Dock:** Aligned directly above the League minimap for Dragon, Baron Nashor, Voidgrubs, Blue Buff, and Red Buff timers.
- **Scoreboard Team Gold Lead:** Shows real-time team gold lead/deficit and lane matchup difference right at the top center.
- **CS Diff Tracker:** Discreet CS score and CS/min pill beside your game clock with live matchup comparison.
- **Optimal Skill Path:** Skill indicator showing the next recommended ability level-up.
- **Instant Hotkeys:** Hit <kbd>Alt + ~</kbd> or <kbd>Ctrl + Space</kbd> anytime to toggle HUD visibility.

### 4. 📊 Profile & Ranked History
- Live rank, division, and LP tracking.
- Win-rate statistics, recent match performances, and MVP/ACE badges.

---

## 📊 Performance Benchmark

| Metric | Commercial Apps / Overwolf | Porofessor / Mobalytics | ⚡ **RiftCore 2.0 (Hybrid)** |
| :--- | :---: | :---: | :---: |
| **Idle Companion RAM** | ~800 MB – 1.4 GB | ~600 MB – 950 MB | **~40 MB** *(Rust)* |
| **In-Game Overlay RAM** | ~1.2 GB+ persistent | ~850 MB persistent | **~300 MB** *(freed on match end)* |
| **Overlay Transparency** | Often borders/artifacts | Washed out or layered | **100% DirectComposition Alpha** |
| **Startup Time** | 4.5 – 9.0s | 3.5 – 6.0s | **< 0.1s** |
| **Third-Party Ads** | Heavy (video/banners) | Heavy (banners/promos) | **ZERO (0)** |
| **CPU Usage** | 3 – 8% background | 2 – 6% background | **< 0.1%** |

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
