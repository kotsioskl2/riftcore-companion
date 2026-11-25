# RiftCore Companion — AI Project State

> Persistent handoff for OpenCode and the RiftCore engineering team. Update after each meaningful, verified work wave.

## Immediate product priorities
1. Repair and polish the native Windows in-game overlay.
2. Substantially improve the native desktop UI/UX.
3. Preserve existing working functionality and Riot policy compliance.

## Verified repository facts (README)
- Native Windows desktop companion built with Rust and egui.
- Main native application: `riftcore-native/`.
- README describes a native HUD and separate main companion window.
- wgpu is used for rendering.
- Legacy TypeScript under `src/` is described as a migration reference, not the active shipped UI.
- Build entry points include `build_rust.bat` and `build_all.bat`.
- Portable executable is staged under `RiftCore_Rust_Portable\\RiftCore.exe`.
- These documentation facts do not prove current runtime behavior.

## User-confirmed product pain points
- The overlay looks and behaves poorly; highest-priority engineering issue.
- The main UI is visually poor and needs a substantial redesign, not a superficial theme change.
- Progress must be visible and concise.
- Repeating audits and agent setup instead of implementing product fixes is unacceptable.

## Mandatory constraints
- No game-memory reading, injection, or anti-cheat bypass.
- No enemy spell/ultimate cooldown tracking.
- No unsupported Mayhem rankings.
- Keep active-game LiveBuildAdvice and CoachInsight disabled pending policy review.
- Preserve post-game analytics and coaching.
- Use permitted, documented Riot API/data sources only.
- Preserve wgpu while evaluating overlay alternatives.
- Any Win32OverlayHost proposal must first be a proof of concept with a fallback and explicit verification of alpha transparency, click-through, focus, mixed DPI, monitor changes, and GPU performance.
- Do not remove legacy binaries until dependencies are verified.
- Do not delete user files, clean drive F:, rewrite Git history, or perform destructive migrations without approval.

## OpenCode agents
Project agent definitions are in `.opencode/agents/*.md`:
- `architect`: primary coordinator and integrator.
- `rust-engineer`: Rust/native app/backend/performance.
- `overlay-engineer`: Windows HUD/rendering/input/DPI.
- `frontend-engineer`: egui UI/UX redesign.
- `riot-compliance`: read-only policy and API review.
- `qa-engineer`: independent verification and regressions.
- `release-security-engineer`: build, updater, installer, and secrets.

Presence of agent files in Git does not prove runtime discovery or invocation. Verify with `opencode agent list` and a small real delegation test.

## OpenCode project configuration
- File: `opencode.jsonc`.
- Default model: `riftcore/gpt-oss-20b`.
- Default agent: `architect`.
- OpenAI-compatible endpoint: `http://127.0.0.1:8000/v1` (valid only inside the AUP session where llama-server runs).
- Model ID configured as `/home/jovyan/models/gpt-oss-20b-MXFP4.gguf`; confirm against live `/v1/models` response if requests fail.
- Declared context: 131,072; declared output allowance: 8,192.
- Automatic/background compaction enabled, preserve 12,000 recent tokens, 20% context buffer.
- OpenCode's provider and compaction settings use current documented fields. The custom model's declared context/output are fallback metadata: OpenCode cannot verify these values itself.
- Runtime context acceptance, full-length KV cache allocation, tool-calling reliability, and long-context quality still need live validation on AUP.
- Do not kill/restart the working llama-server merely to adjust OpenCode settings.

## AUP runtime status — historical user screenshots
- GPT-OSS-20B returned HTTP 200 at `http://127.0.0.1:8000/health`.
- llama.cpp launcher was shown with `-c 131072`, Q8 K/V cache, port 8000.
- These screenshots establish that server startup succeeded at that time, not that 128K tokens were actually exercised end-to-end.
- Treat any AUP session restart as a possible loss of running processes; model files and repository clone are separate persistent artifacts.

## Git branch and sync status
- AI setup branch: `ai/riftcore-opencode-team-2026-11-24`, created from `main`.
- This branch contains the project instructions, six Markdown agent definitions, initial audit note, state file, and OpenCode config.
- Do not merge this branch into `main` automatically; review changes and use a PR.
- User reported a local Windows commit `60c779b` on branch `phase3/integration` for 22 files. The local commit succeeded. Its push to GitHub has not yet been confirmed by the remote branch list.
- Line-ending notices saying LF will be converted to CRLF are Git normalization warnings, not commit failures.
- Once `phase3/integration` is pushed, inspect its diff and compare with this AI setup branch before merging or cherry-picking.

## Engineering workflow
1. State the current task and owner briefly.
2. Inspect only relevant code and current Git state; don't rerun completed audits without a reason.
3. Delegate bounded tasks to the relevant specialist agents.
4. Prevent concurrent agents from editing overlapping files; use separate worktrees when appropriate.
5. Make small, complete changes.
6. Add/update tests and run relevant checks.
7. For visual work, inspect actual Windows screenshots; compilation alone is not visual verification.
8. Review `git diff`; don't claim tests passed unless they ran.
9. Update this file after each meaningful work wave.

## Progress format
Keep updates short:
- CURRENT TASK
- AGENT(S)
- FINDING
- FILES CHANGED
- TEST/BUILD RESULT
- NEXT ACTION

## Current work queue
- [ ] Fetch and review user's `phase3/integration` commit after the push is confirmed.
- [ ] Verify OpenCode discovers all six project agents.
- [ ] Run a real read-only delegation smoke test.
- [ ] Trace overlay host/render/input/focus/DPI/lifecycle implementation.
- [ ] Reproduce the top overlay defects on Windows.
- [ ] Implement first overlay fix and preserve fallback.
- [ ] Trace active egui pages and common components.
- [ ] Implement first high-impact UI redesign slice.
- [ ] Build/test and inspect actual Windows screenshots.
- [ ] Update this file with verified results.

## Work log
- 2026-11-25: Project-level OpenCode provider/agent/compaction configuration and persistent handoff docs added on the AI setup branch. No application source code changed on this branch.
