# RiftCore Companion — Initial Audit Notes

**Status:** Preliminary repository orientation only. This is not a completed source-code or runtime audit.

## Confirmed from the repository README
- RiftCore is described as a native Windows companion using Rust and egui.
- The documented native application lives under `riftcore-native/`.
- The README describes champion selector/build/rune features, draft simulation, League Client connection, post-game summaries, TFT catalog/board functionality, a native HUD, mode setup, and settings.
- The README says the old TypeScript UI under `src/` is retained as a migration reference and is not compiled, embedded, or shipped by the native build.
- The README documents `build_rust.bat` and `build_all.bat` as build entry points.
- The README describes the HUD as a transparent, always-on-top native window and states that it is intended for borderless windowed mode.
- The README states that Riot does not provide live tier-list win rates through Data Dragon, and that previous hard-coded percentages are not presented as current stats.
- The README says enemy cooldown tracking and old Mayhem card rankings are not part of the native build.

## User-reported product issues (not yet independently verified)
- The overlay currently looks and behaves badly.
- The main UI is not visually satisfactory and needs a substantial redesign.

## Required source/runtime investigations
1. Trace the current overlay host, render loop, window styles, alpha composition, input handling, focus/z-order, game-window tracking, tray lifecycle, and DPI behavior.
2. Trace the active egui screen/navigation/component structure and identify the highest-impact UX problems.
3. Trace League Client and Live Client Data integrations against documented behavior and applicable Riot policies.
4. Inspect build scripts, Cargo configuration, updater verification, installer, CI, dependencies, and secret handling.
5. Run existing automated tests/builds and capture actual Windows screenshots for visual review.

## Policy safeguards to verify
- No game-memory reading, injection, or anti-cheat bypass.
- No enemy spell/ultimate cooldown tracking.
- No unsupported Mayhem rankings.
- Keep active-game LiveBuildAdvice and CoachInsight disabled pending policy review.
- Preserve compliant post-game analytics/coaching.
- Use documented, permitted Riot APIs/data only.

## Evidence discipline
Do not promote README claims or user-reported symptoms into verified runtime findings without source inspection, reproducible tests, or visual evidence. Record file paths, symbols, commands, and results for every confirmed issue.
