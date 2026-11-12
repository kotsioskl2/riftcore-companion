# RiftCore Companion

RiftCore is a Windows League of Legends companion with a fully native Rust desktop UI and in-game HUD. The application uses Rust and egui for its windows and rendering. It does not launch Electron or embed a browser engine.

## Features

- Searchable champion selector with role builds, skill order, item sets, full rune page editor and spell import.
- Per-role blind, counter and pocket picks, plus favorite champions.
- Local 5v5 draft simulator, live champ-select sync and qualitative frontline, damage and utility coverage hints.
- League Client connection, queue status, ready-check acceptance, current ranked profile and local match history.
- Riot ID lookup for ranked entries, mastery and recent matches when a user adds their Riot developer API key.
- Post-game match summaries and item breakdowns.
- TFT Set 18 catalog with unit, trait, item and augment images, a saved manual board, and clearly marked Set 13 composition, recipe and roll-odds archives.
- Native HUD with the live game clock, scoreline, CS pace, current gold, ability ranks, inventory, trinket reminder and live objective timers.
- Configurable manual ARAM relic timers.
- Mode setup pages for Summoner's Rift, ARAM, Arena, URF and One for All, with explicit rune/spell imports.
- Global shortcuts: **Alt + ~** or **Ctrl + Space** toggles the HUD; **Alt + Shift + ~** toggles click-through input.
- Native settings for queue automation, API region/key, overlay behavior, champion pool and saved TFT board.
- GitHub release checks with in-app download, safe executable replacement and restart when an update package is available.
- Obsidian styling with cyan, crimson and gold accents, matching RiftCore's previous interface.
- Offline champion portraits, TFT icons and the RiftCore brand mark embedded in the native executable.

## Build

Install the current stable Rust toolchain. The build script uses Visual Studio Build Tools when available, or MinGW-w64 with the `stable-x86_64-pc-windows-gnu` Rust toolchain otherwise. NSIS is needed only to build the Windows installer.

Build and stage the native app:

```cmd
build_rust.bat
```

Build the portable app, installer and ZIP package:

```cmd
build_all.bat
```

The app executable is staged at `RiftCore_Rust_Portable\RiftCore.exe`.

The checked-in TFT icons are already prepared for building. To refresh them from Data Dragon, run `node scripts/download_tft_icons.mjs` and then `powershell -NoProfile -ExecutionPolicy Bypass -File scripts/optimize_tft_icons.ps1` before building.

## Architecture

```text
riftcore-companion/
├── riftcore-native/
│   ├── src/main.rs       # Native window and application entry point
│   ├── src/app.rs        # Rust UI, settings, navigation and HUD
│   ├── src/backend.rs   # League Client and Live Client Data integration
│   └── Cargo.toml
├── installer/            # NSIS installer
├── public/               # App and installer icons
├── all_champions.json    # Champion directory
├── build_rust.bat
└── build_all.bat
```

The former TypeScript interface under `src/` is retained as a migration reference only. It is not compiled, embedded or shipped by the native build.

Champion portraits, TFT catalog data and icons use Riot Games Data Dragon version 16.22.1 and are embedded so the portable app does not need to download game art at runtime. See `riftcore-native/assets/ATTRIBUTION.txt` for the asset source and ownership note.

Riot does not provide live tier-list win rates in Data Dragon, so the app does not show the previous web app's hard-coded percentages as current stats. Set 13 TFT guides remain archived. Enemy cooldown tracking is not included: Riot's [League policy](https://developer.riotgames.com/docs/lol) specifically excludes manual or automatic enemy ultimate tracking. The old Mayhem card rankings and patch-synced tier rates are not part of the native build. Releases without the self-update executable open the release page's setup download as a fallback.

The HUD uses its own transparent, always-on-top native window, so the main companion remains open while the overlay is shown. It follows League's visible game window, hides when the game is minimized or another app is in front, and is intended for borderless windowed mode. Exclusive fullscreen support is not promised. The companion uses the League Client for lobby actions and the Live Client Data API for in-game data. Riot ID scouting requires a user-supplied developer key; the key is saved in the user's local settings file.

## License

RiftCore is released under the MIT License. See [LICENSE](LICENSE).

RiftCore Companion is not endorsed by Riot Games. League of Legends and its associated properties are trademarks of Riot Games, Inc.
