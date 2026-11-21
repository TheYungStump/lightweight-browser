# Project status

| Phase | Status | Branch / PR | Notes |
|---|---|---|---|
| Spec | **v0.3 merged** (2026-11-20): positioning, never-lose-work, AI slider, onboarding, spaces; spec review 2 folded in | `spec-v0.3`, PR #5 (merged) | DEC-008 to DEC-014, from `docs/research/2026-11-browser-positioning.md` and `docs/reviews/spec-review-2.md`. v0.2 had the owner's go-ahead to implement (2026-11-18) |
| 0a: Repo foundation | **Done** (merged) | `phase-0a-foundation`, PR #4 (merged) | Workspace (7 stub crates), tooling, cargo-deny, LICENSE (FSL-1.1-ALv2), CLA (approved), CONTRIBUTING, issue templates, 3-OS CI. See DEC-006 |
| 0b: Windows engine spike | **In progress**: automated checklist items done; owner hands-on items pending (Google sign-in, DRM services, 150% HiDPI, overlay flicker, optional elevated network capture) | `phase-0b-windows-spike` | Report: `docs/spikes/phase0b-windows.md` (provisional **GO**). DEC-015 to DEC-019. Checklist: `docs/testing/phase-0b.md` |
| 0c: macOS + Linux spike | Not started | — | Hands-on macOS checks wait for the MacBook |
| 0d: Memory probe + bench v0 + baselines | Not started | — | |
| 1a / 1b / 1c / 1d: Core browser | Not started | — | Split in DEC-014 (1c = engine-event UIs, 1d = macOS + Linux) |
| 2a / 2b / 2c: Memory management | Not started | — | Split in DEC-014 (2b = capture and restore, 2c = budget + dashboard) |
| 3a / 3b: Built-in features | Not started | — | |
| 4a / 4b / 4c / 4d: Polish & tester release | Not started | — | Split in DEC-014 (4c = onboarding + importers, 4d = release engineering) |
| 5+ | Not started | — | |

## Current handoff notes
_Implementation sessions: write what you finished, what's half-done, and what the next session should do first._

**Phase 0b (2026-11-21):** spike in `spikes/engine-spike/` (standalone; `cargo run --release` from that folder). Every SPEC §10 0b checklist item was tested except the ones that need the owner, and results are in `docs/spikes/phase0b-windows.md`.
- **Done:** multi-view + shared environment, suspend/resume, memory target, audio/mute, process→frame mapping, request interception incl. workers, ad-block latency and memory, local opener popups (normal + private), accelerators, context menu, permissions, fullscreen, cert errors, basic auth, screen capture, isolated world (host = main world; extension = isolated, DEC-015), POST detection, custom protocol + blocking, overlay transparency, empty-window memory, crash-dump locality, EME key systems, Job Object/EcoQoS, non-admin network observation of the telemetry controls.
- **Pending (owner, `docs/testing/phase-0b.md` A1–A5):** real Google sign-in popup, Netflix/Disney+/Spotify playback + resolution, 150% scaling and monitor drag, overlay flicker, optional elevated pktmon/DNS capture (also tests the `ExperimentationAndConfigurationServiceControl` policy). Fill the ⏳ rows of the report from those results, then finish the summary and open the PR.
- Spike data (profiles, logs, filter lists, screenshots, network CSVs) lives under `%LOCALAPPDATA%\lwb-spike`, never in the repo.
- **For 1a:** create the environment ourselves and pass it with `with_environment` (DEC-017); register request filters before the first navigation (DEC-018); `TrySuspend` needs a hidden view.

**Spec v0.3 (2026-11-20):** documentation only, on `spec-v0.3` (PR #5). Applies the owner's decisions from the positioning research: headline and §1 rewrite, comparison against competitors' most aggressive memory savers, form/scroll capture and restore (§7.5), a 500-tab restore test, tab search, auto-archive, onboarding, the AI slider (§8.8), spaces as 5a and extensions as 5b, new 0b/0c checklist items, new risks, and D10–D16 (D9 widened). Spec review 2 (`docs/reviews/spec-review-2.md`) is folded in (DEC-014), including restore-safety and screenshot rules and the 1b/1c/1d, 2a/2b/2c and 4a–4d phase splits. The research report is committed at `docs/research/2026-11-browser-positioning.md`.
- **For 0b:** start from `main` after this merges. The §10 Phase 0b checklist gains WebView2 telemetry, DRM playback and Job Object/EcoQoS items, and `docs/prompts/phase-0b.md` already says to cover every checklist item.

**Phase 0a (2026-11-18):** finished; nothing half-done. `docs/testing/phase-0a.md` was run on Windows: the owner did every step except the lint check, which the implementation session ran (the `unsafe_op_in_unsafe_fn` violation fails with E0133, then reverted).
- The workspace builds with toolchain `1.100.0` (pinned exactly, DEC-006). Each crate has a placeholder module and one test. `lwb-app` and `lwb-bench` print a version line.
- Release `lwb-app.exe` stub (Windows, 1.100.0): 126 KB. A rough baseline for the §4 size budget, which 0d starts tracking in CI.
- Owner setup done (2026-11-19): `docs/CLA.md` approved, CLA Assistant enabled, "Protect main" ruleset active (PR required, 0 approvals, squash only, required checks `windows-latest`, `macos-latest`, `ubuntu-latest`, `cargo-deny`, `license/cla`, branch must be up to date), head branches auto-deleted.
- Review: `docs/reviews/phase-0a-review.md` (on `main` via the spec v0.3 PR), launched as a separate session per SPEC §14 step 3 (DEC-007). Verdict: approve with changes (0 blocking, 4 should fix, 9 nice to have). The code and doc findings are fixed on PR #4. Owner checklist run on Windows (2026-11-19), CLA signed, and the Gist's "draft" note removed. Still open: turn on private vulnerability reporting.
- **For 0d:** the release profile's `panic = "abort"` applies to `lwb-bench` too. A panic would then skip `Drop`, leaving launched browsers and temp profiles behind. Either give the harness `[profile.bench] inherits = "release"` + `panic = "unwind"` and build with `--profile bench`, or make its cleanup independent of `Drop`.
- **For the lawyer review (SPEC §5.1):** the CLA dropped the Apache ICLA's reservation-of-rights sentence ("You reserve all right, title, and interest…") and its "Subject to the terms…" lead-in. Restore them or list the removal under "Changes". Any text change also means updating the Gist.
- For 0c: wry's README confirms `build_as_child` is X11-only on Linux and recommends `build_gtk` for Wayland, which matches SPEC §6.1's (verify) note.
