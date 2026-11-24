# RiftCore Companion — Engineering Instructions

## Product priorities
The immediate product priorities are:
1. Repair and polish the native Windows in-game overlay.
2. Redesign the main desktop UI to feel cohesive, professional, and production-quality.
3. Preserve existing functionality while making those improvements.

Do not spend cycles repeating repository audits or setting up tooling when an actionable, approved engineering task is available.

## Architecture facts
- The shipped application is a native Windows desktop application built with Rust and egui.
- The native application and HUD live under `riftcore-native/`.
- The legacy TypeScript interface under `src/` is a migration reference and is not the active shipped UI.
- The application uses wgpu for rendering. Preserve it while evaluating overlay alternatives.
- Consult the current source before relying on this summary; do not assume README details prove current runtime behavior.

## Overlay engineering
- Investigate transparency/alpha composition, window styles, positioning, z-order, focus, click-through input, DPI scaling, multi-monitor behavior, show/hide lifecycle, tray integration, game-window detection, flicker, and frame time.
- Keep wgpu unless a measured, documented reason justifies a change.
- Evaluate a `Win32OverlayHost` behind `OverlaySurfaceHost` as a proof of concept before considering replacement of the current host.
- Verify the proof of concept on the target Windows environment for alpha transparency, click-through, focus, mixed DPI, monitor changes, and GPU performance.
- Preserve a working fallback until the replacement passes verification.
- Do not use color-key transparency as a shortcut; investigate proper per-pixel alpha composition.
- Avoid global scaling hacks. Derive layout from DPI and measured window geometry.

## UI/UX engineering
- Treat the current UI quality as a product-level issue, not a request for a superficial theme change.
- Inspect all active screens and reusable components before redesigning.
- Improve information hierarchy, typography, spacing, navigation, consistency, empty/loading/error states, and scaling.
- Preserve the established RiftCore identity where appropriate (obsidian/dark surfaces with restrained cyan, crimson, and gold accents), but prioritize readability and visual discipline.
- Prefer reusable egui components and design tokens over one-off styling.
- Capture or otherwise inspect actual rendered screens when a visual test environment is available. A successful compile is not proof of visual quality.

## Riot policy and game integrity — mandatory
- Never read game memory, inject into the game, or bypass Vanguard/anti-cheat.
- Do not implement enemy spell/ultimate cooldown tracking.
- Do not add unsupported Mayhem rankings.
- Keep active-game LiveBuildAdvice and CoachInsight disabled pending policy review.
- Preserve post-game analytics and coaching.
- Use documented, permitted Riot APIs and data sources only.
- Treat Riot product registration and policy compliance as release requirements.
- Do not infer that an undocumented endpoint is permitted merely because it works.

## Security and updater
- Never commit API keys, tokens, or production secrets.
- If a key may have been exposed, recommend rotation/revocation; do not copy it into logs or docs.
- Use OS-protected local secret storage where appropriate (Windows DPAPI) and server-side secrets for production services.
- Updater verification must fail closed: verify SHA-256 and a trusted signature/manifest before execution; reject unverified downloads and unsafe redirects.
- Retain a browser-download fallback if in-app update verification cannot be completed.
- Do not remove legacy binaries until all references and runtime dependencies are verified.

## Build and repository safety
- Inspect `git status` before editing and preserve unrelated user changes.
- On Windows, use `D:\RiftCore-CargoTarget` as the preferred Cargo target directory when configured; ensure `build_rust.bat` respects `CARGO_TARGET_DIR`.
- Do not clean drive F:.
- Do not delete user files, rewrite Git history, force-push, or run destructive migrations without explicit approval.
- Use separate Git worktrees for independent concurrent writers when practical. Do not let agents edit the same files concurrently.
- The Architect integrates work only after reviewing diffs and resolving conflicts.

## Engineering workflow
1. Restate the task in one sentence and identify the owner.
2. Inspect only the relevant source and current Git state; do not rerun completed audits without a reason.
3. Delegate bounded work to the relevant specialist subagents when useful.
4. Make the smallest coherent change that addresses the issue.
5. Add/update tests and run relevant checks.
6. Have QA review behavior and regressions independently.
7. Inspect `git diff`; do not claim a test passed unless it actually ran.
8. Update `docs/AI_PROJECT_STATE.md` at the end of each meaningful work wave.

## Progress format
Keep updates short and visible:
- CURRENT TASK
- AGENT(S) WORKING
- FINDING
- FILES CHANGED
- TEST/BUILD RESULT
- NEXT ACTION

Never claim subagent delegation unless the subagent tool was actually invoked. Never claim visual success based only on compilation.
