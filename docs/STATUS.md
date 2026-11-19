# Project status

| Phase | Status | Branch / PR | Notes |
|---|---|---|---|
| Spec | v0.2, owner gave the go-ahead to implement (2026-11-18) | — | Review 1 incorporated. The spec header still says DRAFT; the orchestrator updates it |
| 0a: Repo foundation | **Done**, ready to merge (CI green; review: approve with changes, fixed; owner checklist run) | `phase-0a-foundation`, PR #4 | Workspace (7 stub crates), tooling, cargo-deny, LICENSE (FSL-1.1-ALv2), CLA (approved), CONTRIBUTING, issue templates, 3-OS CI. See DEC-006 |
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

**Phase 0a (2026-11-18):** finished; nothing half-done. `docs/testing/phase-0a.md` was run on Windows: the owner did every step except the lint check, which the implementation session ran (the `unsafe_op_in_unsafe_fn` violation fails with E0133, then reverted).
- The workspace builds with toolchain `1.100.0` (pinned exactly, DEC-006). Each crate has a placeholder module and one test. `lwb-app` and `lwb-bench` print a version line.
- Release `lwb-app.exe` stub (Windows, 1.100.0): 126 KB. A rough baseline for the §4 size budget, which 0d starts tracking in CI.
- Owner setup done (2026-11-19): `docs/CLA.md` approved, CLA Assistant enabled, "Protect main" ruleset active (PR required, 0 approvals, squash only, required checks `windows-latest`, `macos-latest`, `ubuntu-latest`, `cargo-deny`, `license/cla`, branch must be up to date), head branches auto-deleted.
- Review: `docs/reviews/phase-0a-review.md` on branch `review/phase-0a`, launched as a separate session per SPEC §14 step 3 (DEC-007). Verdict: approve with changes (0 blocking, 4 should fix, 9 nice to have). The code and doc findings are fixed on PR #4. Owner checklist run on Windows (2026-11-19), CLA signed, and the Gist's "draft" note removed. Still open: turn on private vulnerability reporting.
- **For 0d:** the release profile's `panic = "abort"` applies to `lwb-bench` too. A panic would then skip `Drop`, leaving launched browsers and temp profiles behind. Either give the harness `[profile.bench] inherits = "release"` + `panic = "unwind"` and build with `--profile bench`, or make its cleanup independent of `Drop`.
- **For the lawyer review (SPEC §5.1):** the CLA dropped the Apache ICLA's reservation-of-rights sentence ("You reserve all right, title, and interest…") and its "Subject to the terms…" lead-in. Restore them or list the removal under "Changes". Any text change also means updating the Gist.
- For 0c: wry's README confirms `build_as_child` is X11-only on Linux and recommends `build_gtk` for Wayland, which matches SPEC §6.1's (verify) note.
