# Prompt: PR review session

Paste the following into a fresh local Claude Code session in this repo, replacing `<N>` and `<id>`.

---

You are reviewing pull request #<N> (Phase <id>) of a low-memory desktop browser written in Rust. Check out the PR branch. Read `CLAUDE.md`, `docs/SPEC.md` (the sections for this phase in §10, and every section the phase touches), `docs/STATUS.md` and `docs/DECISIONS.md`.

Review the PR as a senior Rust/systems engineer. Don't fix anything yourself. Check:
1. **Acceptance criteria:** go through every acceptance criterion and checklist item for this phase in SPEC §10. For each: met / partly met / not met, with evidence (file:line, test name, CI run, report section).
2. **Spec conformance:** any divergence from SPEC that isn't recorded in DECISIONS.md is a finding.
3. **Correctness:** bugs, race conditions, error handling, resource/memory leaks (webviews, COM objects, handles), panics on untrusted input.
4. **Security:** SPEC §6.6, i.e. view kinds and privilege, IPC origin checks, page-channel messages treated as untrusted, no `lwb://`/IPC exposure to web content, filter-list safety.
5. **Memory:** new dependencies and their size/RAM cost; anything held in memory that doesn't need to be; anything that defeats hibernation.
6. **Rules from CLAUDE.md:** `lwb-core` purity, no wry/platform types in `lwb-engine`'s public API, `// SAFETY:` comments, private-API policy, tests for policy logic.
7. **Tests and CI:** CI green on all three OSes; tests meaningful, not just present; the manual checklist `docs/testing/phase-<id>.md` exists and is runnable.

Write `docs/reviews/phase-<id>-review.md`: a verdict (approve / approve with changes / changes required), then findings as **Blocking**, **Should fix**, **Nice to have**, each with location, problem, and suggested fix. Commit it on a branch named `review/phase-<id>` and push.
