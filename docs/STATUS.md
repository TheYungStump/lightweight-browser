# Project status

| Phase | Status | Branch / PR | Notes |
|---|---|---|---|
| Spec | v0.2, owner gave the go-ahead to implement (2026-11-18) | — | Review 1 incorporated. The spec header still says DRAFT; the orchestrator updates it |
| 0a: Repo foundation | **Done**, PR open, awaiting review | `phase-0a-foundation` | Workspace (7 stub crates), tooling, cargo-deny, LICENSE (FSL-1.1-ALv2), CLA draft, CONTRIBUTING, issue templates, 3-OS CI. See DEC-006 |
| 0b: Windows engine spike | Ready to start after approval | `phase-0b-windows-spike` | Prompt: `docs/prompts/phase-0b.md`. Can run in parallel with 0a |
| 0c: macOS + Linux spike | Not started | — | Hands-on macOS checks wait for the MacBook |
| 0d: Memory probe + bench v0 + baselines | Not started | — | |
| 1a / 1b / 1c: Core browser | Not started | — | |
| 2a / 2b: Memory management | Not started | — | |
| 3a / 3b: Built-in features | Not started | — | |
| 4a / 4b: Polish & tester release | Not started | — | |
| 5+ | Not started | — | |

## Current handoff notes
_Implementation sessions: write what you finished, what's half-done, and what the next session should do first._

**Phase 0a (2026-11-18):** finished; nothing half-done.
- The workspace builds with toolchain `1.100.0` (pinned exactly, DEC-006). Each crate has a placeholder module and one test. `lwb-app` and `lwb-bench` print a version line.
- Release `lwb-app.exe` stub (Windows, 1.100.0): 126 KB. A rough baseline for the §4 size budget, which 0d starts tracking in CI.
- Owner to-dos (by hand): approve `docs/CLA.md`, enable CLA Assistant, add branch protection on `main` requiring the CI checks. Steps are in the 0a PR description.
- For 0c: wry's README confirms `build_as_child` is X11-only on Linux and recommends `build_gtk` for Wayland, which matches SPEC §6.1's (verify) note.
