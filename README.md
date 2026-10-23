# ⚔️ RiftCore Companion

<div align="center">

![License](https://img.shields.io/badge/license-MIT-blue.svg)
![Rust](https://img.shields.io/badge/backend-Rust_1.80+-orange.svg?logo=rust)
![React](https://img.shields.io/badge/frontend-React_19_+_TypeScript-61dafb.svg?logo=react)
![RAM Footprint](https://img.shields.io/badge/memory-~38_MB_RAM-brightgreen.svg)
![Ad-Free](https://img.shields.io/badge/ads-ZERO_ADS-crimson.svg)

**The ultra-lightweight, ad-free, high-performance League of Legends companion built in Rust.**

[Features](#-features) • [Performance Comparison](#-performance-benchmark) • [Installation](#-installation) • [Build from Source](#-building-from-source) • [Architecture](#-architecture) • [License](#-license)

</div>

---

## ⚡ Why RiftCore?

Tired of bloated League companion apps (Overwolf, Blitz, Porofessor) consuming **1GB+ of RAM**, injecting intrusive audio/video ads, and slowing down your PC mid-game?

**RiftCore** was engineered from the ground up to solve this:
- **Zero Ads, Zero Trackers:** Pure utility, 100% free and open-source forever.
- **Native Rust Core:** Powered by native Rust (`wry` + `tao` + `tokio`), running directly against the League Client Update (LCU) REST & WebSocket APIs and local game memory port.
- **Microscopic Memory Footprint:** Idles around **~38 MB RAM** (up to 25x less RAM than Electron/Overwolf apps).
- **Minimalist Red & Blue Duality:** Sleek obsidian dark UI with clear chromatic division: **Cyan/Blue (`#00c8ff`)** for ally utility, runes, and matchmaking; **Crimson Red (`#ff2a5f`)** for enemy threats, cooldowns, and jungle objectives.

---

## 🚀 Features

### 1. ⚡ Instant Auto-Accept
- Automatically detects match found queue pops and accepts instantly.
- Plays crisp audio cues and alerts.

### 2. 🔮 1-Click Rune & Summoner Spell Import
- High-winrate meta builds for all 160+ champions across Top, Jungle, Mid, ADC, and Support.
- Imports complete rune trees (Primary, Secondary, Stat Shards) directly into your active League client perk page with one click.
- Auto-assigns recommended Flash + Summoner Spell combinations.

### 3. 🎯 Live Game HUD & Threat Tracker
- Real-time in-game overlay querying local port 2999 Live Client Data.
- Live dragon, Baron Nashor, and Rift Herald objective timers.
- Enemy summoner spell cooldown timers (Flash, Teleport, Ignite, Cleanse).
- Real-time CS/min, gold differentials, and combat stat tracking.

### 4. 📊 Profile & Ranked History
- Live rank, division, and LP tracking.
- Win-rate statistics, recent match performances, and MVP/ACE badges.

### 5. 🪟 Compact Overlay Mode
- Toggle between full dashboard mode (1240x820) and a sleek pinned HUD mode (420x720) that floats unobtrusively beside or on top of your League client.

---

## 📊 Performance Benchmark

| Metric | Blitz.gg / Overwolf | Porofessor / Mobalytics | ⚡ **RiftCore (Rust)** |
| :--- | :---: | :---: | :---: |
| **Idle Memory (RAM)** | ~800 MB – 1.4 GB | ~600 MB – 950 MB | **~38 MB** |
| **Standalone Binary Size** | ~280 MB – 450 MB | ~200 MB – 350 MB | **~11 MB** |
| **Startup Time** | 4.5 – 9.0s | 3.5 – 6.0s | **< 0.4s** |
| **Third-Party Ads** | Heavy (video/banners) | Heavy (banners/promos) | **ZERO (0)** |
| **CPU Usage** | 3 – 8% background | 2 – 6% background | **< 0.1%** |

---

## 📦 Downloads & Installation

### Direct Downloads
| Package | Type | Download Link | Size |
| :--- | :--- | :--- | :--- |
| **Windows Installer** | Standalone Setup | [📥 **Download RiftCore-Setup-1.0.0.exe**](https://github.com/kotsioskl2/riftcore-companion/raw/main/installer_output/RiftCore-Setup-1.0.0.exe) | ~4.7 MB |
| **Portable Package** | ZIP Archive | [📦 **Download RiftCore-Portable-1.0.0.zip**](https://github.com/kotsioskl2/riftcore-companion/raw/main/installer_output/RiftCore-Portable-1.0.0.zip) | ~7.4 MB |
| **Portable Folder** | Extracted Directory | [📁 **Browse RiftCore_Rust_Portable/**](./RiftCore_Rust_Portable/) | Ready to Run |

### Option A: Windows Installer (Recommended)
1. Download [**RiftCore-Setup-1.0.0.exe**](https://github.com/kotsioskl2/riftcore-companion/raw/main/installer_output/RiftCore-Setup-1.0.0.exe).
2. Run the setup and choose your installation directory.
3. Desktop and Start Menu shortcuts with the custom RiftCore emblem will be created automatically.

### Option B: Standalone Portable `.exe`
1. Download [**RiftCore-Portable-1.0.0.zip**](https://github.com/kotsioskl2/riftcore-companion/raw/main/installer_output/RiftCore-Portable-1.0.0.zip) or open the [`RiftCore_Rust_Portable/`](./RiftCore_Rust_Portable/) directory.
2. Double-click `RiftCore.exe` (or `Launch_RiftCore.bat`) to launch instantly with zero installation required.

---

## 🛠️ Building from Source

### Prerequisites
- [Node.js](https://nodejs.org/) (v18+) & `npm`
- [Rust](https://rustup.rs/) (v1.80+ with `x86_64-pc-windows-gnu` or `msvc` toolchain)
- MinGW / GCC (e.g. `w64devkit` or MSVC C++ Build Tools)
- [NSIS](https://nsis.sourceforge.io/) (for compiling the installer)

### 1. Clone the repository
```bash
git clone https://github.com/kotsioskl2/riftcore-companion.git
cd riftcore-companion
```

### 2. Install dependencies & build frontend
```bash
npm install
npm run build:renderer
```

### 3. Compile the Native Rust binary
```bash
cd riftcore-native
cargo build --release
```

### 4. Assemble Portable Release
```bash
# Copy dist web assets and WebView2Loader.dll to release folder
xcopy /E /I /Y "..\dist" "target\release\dist"
```

### 5. Build the Windows Installer
```bash
makensis installer\installer.nsi
```
The compiled installer will be generated in `installer_output\RiftCore-Setup-1.0.0.exe`.

---

## 🏗️ Architecture

```
riftcore-companion/
├── src/                      # Modern React 19 + TypeScript Frontend
│   ├── components/           # Minimalist Red & Blue UI Views
│   │   ├── TitleBar.tsx      # Window controls, live client status pill
│   │   ├── ChampSelectView.tsx # Meta builds, 1-click rune & spell import
│   │   ├── LiveGameOverlay.tsx # Objective & enemy cooldown HUD
│   │   ├── SummonerProfileView.tsx # Ranked LP stats & match history
│   │   └── ChampionTierListView.tsx # Champion tier list & counter picks
│   ├── rustBridge.ts         # Bi-directional IPC bridge to native Rust
│   └── index.css             # Obsidian styling with Electric Cyan & Crimson
├── riftcore-native/          # Native Rust Runtime
│   ├── src/main.rs           # Tokio async engine, LCU REST API client,
│   │                         # wry WebView2 renderer, custom protocol handler
│   └── Cargo.toml            # Dependencies: wry, tao, tokio, reqwest
├── installer/                # NSIS Windows Installer Scripts
│   └── installer.nsi         # Modern UI installer definition
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
