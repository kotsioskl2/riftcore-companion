# RiftCore Companion — AI Project State

> Persistent handoff file for OpenCode and the RiftCore engineering agents. Update this file after each meaningful, verified work wave.

## Product priorities
1. Fix and polish the native Windows in-game overlay.
2. Substantially improve the native desktop UI/UX.
3. Preserve existing working functionality and Riot policy compliance.

## Verified repository facts (from README)
- The product is a native Windows desktop companion built with Rust and egui.
- Main native application: `riftcore-native/`.
- The README describes a native HUD and a separate main companion window.
- wgpu is used for rendering.
- The old TypeScript interface under `src/` is described as a migration reference, not the active shipped UI.
- Build entry points include `build_rust.bat` and `build_all.bat`.
- The portable executable is staged under `RiftCore_Rust_Portable\RiftCore.exe`.

These are documentation facts, not a substitute for verifying current source behavior.

## User-confirmed pain points
- The overlay looks and behaves poorly and is the highest-priority engineering issue.
- The main UI is visually poor and needs a substantial redesign, not a superficial theme change.
- Progress must be visible and concise.
- Repeating audits and agent setup instead of implementing product fixes is unacceptable.

## Mandatory constraints
- No game-memory reading, injection, or anti-cheat bypass.
- No enemy spell/ultimate cooldown tracking.
- No unsupported Mayhem rankings.
- Keep active-game LiveBuildAdvice and CoachInsight disabled pending policy review.
- Preserve post-game analytics/coaching.
- Use permitted, documented Riot API/data sources only.
- Preserve wgpu while evaluating overlay alternatives.
- Any Win32OverlayHost proposal must first be a proof of concept with a fallback and explicit verification of alpha transparency, click-through, focus, mixed DPI, monitor changes, and GPU performance.
- Do not remove legacy binaries until dependencies are verified.
- Do not delete user files, clean drive F:, rewrite Git history, or make destructive migrations without approval.

## Team
- `architect`: primary coordinator and integrator.
- `rust-engineer`: Rust/native app/backend/performance.
- `overlay-engineer`: Windows HUD/rendering/input/DPI.
- `frontend-engineer`: egui UI/UX redesign.
- `riot-compliance`: read-only policy and API review.
- `qa-engineer`: independent verification and regressions.
- `release-security-engineer`: build, updater, installer, and secrets.

## Current status
- [ ] Agent files discovered by the installed OpenCode runtime.
- [ ] Agent delegation smoke test completed.
- [ ] Overlay defect inventory completed against current source.
- [ ] First overlay fix implemented and verified on Windows.
- [ ] UI defect inventory completed against current screens.
- [ ] First UI improvement implemented and visually checked.
- [ ] Relevant tests/builds passed.
- [ ] Actual screenshots reviewed.

## Next actions
1. Open the repository in OpenCode with `architect` selected.
2. Ask the Architect to invoke overlay-engineer and frontend-engineer for bounded investigations in parallel.
3. Have rust-engineer inspect shared native/windowing constraints.
4. Choose one reproducible overlay defect and one high-impact UI screen for the first implementation wave.
5. Assign QA acceptance criteria before implementation and verify results after integration.

## Runtime-specific setup
AUP GPU/server paths, process IDs, VRAM use, and model/API identifiers are runtime-specific and must not be hardcoded in this repository. Preserve the user's existing global model/provider configuration unless it is explicitly inspected and backed up before editing.

## Work log
No application source files have been modified by this setup branch. Add dated entries below only after real work is completed and verified.
