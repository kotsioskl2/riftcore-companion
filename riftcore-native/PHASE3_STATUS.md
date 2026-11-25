# Phase 3 — Modular Integration Checkpoint

## Runtime selection
- Default: legacy monolith (`app::RiftCoreApp`) — unchanged, still the shipped behavior.
- Opt-in: `--modular-ui` or `RIFTCORE_UI=modular` runs `ui::shell::RiftCoreShell`
  (modular `ui::chrome` + `ui::pages`, Path B `Win32OverlayHost`).
- `--legacy-ui` / `RIFTCORE_UI=legacy` forces legacy. CLI flag beats env var.
- If the modular path fails to start (e.g. no DX12/DxgiFromVisual), `main` falls back to legacy.
- If Path B init fails inside the shell, the Path A stand-in keeps lifecycle state but does
  **not draw on screen**; the user is told to restart with `RIFTCORE_UI=legacy`.
- Legacy files (`app.rs`, `backend.rs`, `builds.rs`, `tft_data.rs`) are intentionally retained.

## Verified (Windows 11, release build, isolated APPDATA, no League running)
- Modular shell launches; main window, tray, and hidden Path B HUD window are created
  (no fallback taken).
- Legacy runtime launches.
- Modular UI renders (sidebar, Champ Select, quick tour); champion portraits load lazily.
- Non-default theme (Ember) is applied from saved settings at startup.
- First close shows the close/tray confirmation (matches legacy).

## NOT verified — outstanding
- **Live-game overlay**: placement over the League window, mouse pass-through, Alt+` /
  Alt+Shift+` hotkeys, auto-show on match start / auto-hide on exit.
- **GPU device-loss recovery**: not implemented — the Path B host is not recreated if the
  device or surface is lost mid-session.
- **League client reconnection**: disconnect/reconnect of the LCU while the modular shell runs.
- Live theme switching from the Settings page (code path exists; not exercised at runtime).
- Full graceful shutdown via tray Exit.

## Known data gap
- TFT augment tier labels (Silver/Gold/Prismatic) cannot be confirmed: `assets/data/tft-augments.json`
  has no `tier` field, so the label never renders. Mapping left unchanged.
