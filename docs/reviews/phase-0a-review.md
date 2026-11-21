# Review: Phase 0a, repo foundation (PR #4)

- PR: https://github.com/TheYungStump/lightweight-browser/pull/4 (`phase-0a-foundation` → `main`)
- Reviewed head: `2b61458` ("Allow implementation sessions to launch review sessions; mark CLA approved"), on top of `5db4ff3` ("Phase 0a: repo foundation")
- Reviewed against: `CLAUDE.md`, SPEC §5.1, §6.1, §6.2, §10 Phase 0a, §14, `docs/prompts/phase-0a.md`, `docs/STATUS.md`, `docs/DECISIONS.md` (DEC-001…DEC-007)
- Reviewer had no access to the implementer's session, only the repo, the PR and its CI.

## Verdict: **Approve with changes**

The foundation is solid. The workspace matches SPEC §6.2, CI is green on all three OSes for the latest commit, `cargo deny check` passes, `LICENSE` is the official FSL-1.1-ALv2 text with only the copyright line filled in, and every choice that wasn't fixed by the spec is recorded in DEC-006. Nothing blocks the code. Before merging, the owner should deal with the four **Should fix** items. Three of them are owner actions or a merge-time choice, not code changes: the manual checklist, the CLA status check, and the model name in the commit trailers.

Findings: **0 Blocking, 4 Should fix, 9 Nice to have.**

## What I checked and how

- **CI:** `gh pr checks 4` → `windows-latest`, `macos-latest`, `ubuntu-latest`, `cargo-deny` all **pass** in run [44464064712](https://github.com/TheYungStump/lightweight-browser/actions/runs/44464064712), whose `headSha` is `2b61458`, the PR's latest commit. Logs show `rustc 1.100.0` on all three OSes, 7 unit tests passing per OS, and `advisories ok, bans ok, licenses ok, sources ok` in the deny job. `license/cla` is **pending** (see S2).
- **Locally (Windows, 1.100.0):** `cargo fmt --all --check` clean; `cargo clippy --workspace --all-targets -- -D warnings` clean; `cargo test --workspace` → 7 passed (plus 5 empty doc-test runs); `cargo deny check` (0.21.0) → all ok.
- **LICENSE:** fetched `FSL-1.1-ALv2.template.md` from `getsentry/fsl.software` through the GitHub API and diffed it. The only difference is line 9, `Copyright ${year} ${licensor name}` → `Copyright 2026 TheYungStump`.
- **Dependencies:** `Cargo.lock` lists only the 7 workspace crates. No third-party crates, so nothing to weigh on memory or size.
- **Linux packages:** checked muda's current README and `Cargo.toml`. With the default `gtk3` feature, muda needs only `libgtk-3-dev` (no `libxdo-dev` any more), so DEC-006's claim "nothing else is required" holds.
- **Repo settings (read-only):** labels, private vulnerability reporting and branch protection on `main` (see N4, N5).

## Acceptance criteria and deliverables

SPEC §10 Phase 0a, plus the deliverable list in `docs/prompts/phase-0a.md`.

| # | Item | Status | Evidence |
|---|---|---|---|
| A1 | Workspace skeleton per §6.2, empty crates that compile | **Met** | `Cargo.toml:1-13`: members `lwb-app`, `lwb-core`, `lwb-engine`, `lwb-memprobe`, `lwb-blocker`, `lwb-storage`, `tools/lwb-bench`; `exclude = ["spikes/*"]`. `lwb-app` and `lwb-bench` print a version line (`crates/lwb-app/src/main.rs:3-9`, `tools/lwb-bench/src/main.rs:3-9`). Each library has `placeholder.rs` with one test. `ui/`, `bench/` and `spikes/` don't exist yet, which is fine because git can't hold empty dirs and nothing needs them in 0a. |
| A2 | Edition 2024, `rust-version` = current stable | **Met** | `Cargo.toml:17-18`; `rust-toolchain.toml:4` pins `1.100.0` (CI log: `rustc 1.100.0 (0ef1bc386 2026-11-09)`). Exact pin recorded in DEC-006. |
| A3 | `[workspace.lints]`: deny `unsafe_op_in_unsafe_fn`, clippy `all` warn | **Met** | `Cargo.toml:23-27`; every crate has `[lints] workspace = true`. Checklist step "Lints actually apply" covers the deny. |
| A4 | Release profile tuned for size/memory, with rationale | **Met** | `Cargo.toml:31-36`; rationale table in the PR description and `CONTRIBUTING.md:64-73`. See N1, N2 for two consequences worth writing down. |
| A5 | `rustfmt.toml`, `.editorconfig`, pinned `rust-toolchain.toml` | **Met** | All present; rustfmt options are stable ones. `.gitattributes` adds LF normalisation, a sensible extra. |
| A6 | `deny.toml`: allow-list, advisories, duplicate versions as warnings, GPL/AGPL denied | **Met** | `deny.toml:21-31` (allow-list), `:7-11` (advisories), `:43-49` (bans). LGPL omission recorded in DEC-006. `Apache-2.0 WITH LLVM-exception` was added beyond the prompt's list (N8). CI deny job green. |
| A7 | `LICENSE` = official FSL-1.1-ALv2 text, licensor + 2026 | **Met** | Diff against upstream template above. |
| A8 | `CONTRIBUTING.md` (build, PR rules, CLA), `docs/CLA.md`, README licensing section | **Met** | All present and accurate. CLA marked owner-approved in `docs/CLA.md:3`. See N3, N7. |
| A9 | CLA Assistant config / explain how to enable it | **Met (deviation recorded)** | No committed config, recorded in DEC-006. The PR description gives correct step-by-step instructions, including the Gist requirement. STATUS says it's enabled, and the `license/cla` check appears on the PR. But see S2 and N3. |
| A10 | Issue templates: bug, feature, tester feedback | **Met** | `.github/ISSUE_TEMPLATE/{bug_report,feature_request,tester_feedback}.yml`, `config.yml` disables blank issues. Tester feedback covers OS, version, RAM and memory observations. See N4, N5. |
| A11 | CI: 3-OS matrix, fmt, clippy, build, test; cargo-deny job on Linux; WebKitGTK packages; pinned actions; PR + push-to-main triggers; cancel superseded runs | **Met** | `.github/workflows/ci.yml`. `permissions: contents: read` and `pull_request` (not `pull_request_target`) are the right security defaults. See N6, N9. |
| A12 | CLAUDE.md build/test commands | **Met** | `CLAUDE.md:25-41`. Commands match what CI runs. |
| A13 | STATUS.md: 0a marked done, with summary | **Met, slightly premature** | `docs/STATUS.md:6`, handoff note at `:19-24`. "Done" is stated before the manual checklist has been run (S1). |
| A14 | Manual checklist `docs/testing/phase-0a.md` exists and is runnable | **Met** | Steps are concrete and quick. I ran its automatable steps locally and they pass. |
| **AC1** | **CI green on all three OSes** | **Met** | Run 44464064712 on `2b61458`. |
| **AC2** | **`cargo deny check` passes** | **Met** | CI deny job and a local run. |
| DoD | §10 definition of done: manual checklist run on Windows | **Not yet** | PR test plan: "Manual checklist … run by the owner" unchecked (S1). |

## Spec conformance

Every divergence I found is recorded:
- Exact toolchain pin vs "MSRV = latest stable at the start of each phase" (§6.1): DEC-006.
- No LGPL entry on the allow-list (prompt said "LGPL only for system libraries"): DEC-006, with sound reasoning (cargo-deny never sees system libraries).
- No committed CLA Assistant config (§10 0a says "CLA Assistant config"): DEC-006.
- SPEC §14 step 3 and `_kickoff.md` step 4 edited to let the implementer launch the review session: DEC-007.

The only unrecorded detail is the extra `Apache-2.0 WITH LLVM-exception` license (N8), which is harmless.

## Findings

### Blocking

None.

### Should fix

**S1. Manual checklist not yet run on Windows, though STATUS already says "Done"**
- Location: PR #4 test plan (last box unchecked); `docs/STATUS.md:6` ("**Done**, CI green, in review") and `:19` ("finished; nothing half-done").
- Problem: SPEC §10 says a sub-phase is done only once `docs/testing/phase-<id>.md` "has been run on Windows". The checklist also has repo-level steps that CI can't cover: exactly three issue templates offered, Markdown links working on GitHub, and a second push cancelling the in-progress run.
- Fix: the owner runs `docs/testing/phase-0a.md` on Windows before merging and ticks the PR box. Optionally reword the STATUS row to "Done pending owner checklist" until then.

**S2. `license/cla` status check is pending on the PR**
- Location: `gh pr checks 4` → `license/cla pending … Contributor License Agreement is not signed yet.`
- Problem: the PR's only commit author is the owner, and the check still says unsigned. Either the owner hasn't signed or allow-listed themselves in CLA Assistant, or CLA Assistant is also counting the `Co-Authored-By: … <noreply@anthropic.com>` trailer on both commits (S3), an identity that can never sign. If the owner later makes `license/cla` a required check (as the PR's own to-do suggests), this PR, and every future PR with that trailer, will be stuck.
- Fix: in CLA Assistant, sign as the owner or add the owner to the allow-list, then re-check the PR. If it stays pending, open the cla-assistant.io link on the check to see which identity it's waiting on. If it's the co-author address, resolving S3 fixes it too, since a squash merge without the trailer satisfies it for the merged commit. For future phases, decide once whether session commits carry a co-author trailer at all.

**S3. Commit messages contain an AI model name, against a CLAUDE.md rule**
- Location: both PR commits, `5db4ff3` and `2b61458`, end with a `Co-Authored-By:` trailer naming a specific model and version.
- Problem: CLAUDE.md: "Don't include AI model names or identifiers in commits, code comments, or docs." Code, comments and docs in the PR are clean; only the commit trailers break the rule.
- Fix: no force-push needed. **Squash-merge** PR #4 and edit the squash message to drop the trailer (or replace it with a trailer that names no model, if the owner wants attribution). For future sessions, add the explicit wording "no model names in `Co-Authored-By` trailers either" to CLAUDE.md or the kickoff prompt, because the default session tooling adds this trailer automatically.

**S4. CLA Assistant actually serves a Gist, not `docs/CLA.md`, so the two can drift**
- Location: `docs/DECISIONS.md` DEC-006 ("The app is configured in its web UI and points at `docs/CLA.md`"); `docs/CLA.md:3`.
- Problem: cla-assistant.io signs contributors against a GitHub Gist, as the PR description correctly says. DEC-006 says it points at `docs/CLA.md`, which is inaccurate. More importantly, the text contributors actually sign is the Gist, so any later edit to `docs/CLA.md` that isn't copied to the Gist (or the reverse) means the repo shows one agreement while people sign another. That matters for the relicensing right the CLA exists to secure (SPEC §5.1).
- Fix: correct DEC-006 to say the Gist is the signed text and `docs/CLA.md` is its mirror. Add a line under the status note in `docs/CLA.md` naming the Gist URL and saying both must change together (CLA Assistant re-asks signers when the Gist changes). Optionally record the Gist revision that was approved.

### Nice to have

**N1. `strip = true` makes panic-log backtraces unsymbolised**
- Location: `Cargo.toml:36`; rationale in `CONTRIBUTING.md:73` ("Crash diagnostics come from our panic log rather than symbols in the binary").
- Problem: the §8.6 panic hook will still record the message and `file:line` (those are compiled in), but a `std::backtrace::Backtrace` captured in a stripped binary is addresses only. The rationale reads as if diagnostics lose nothing.
- Fix: no change needed now. Reword the rationale to say backtraces are address-only. When §8.6 is built (Phase 4), consider `split-debuginfo = "packed"` and keeping the PDB/dSYM/debug files as private release artifacts for offline symbolication. That keeps the shipped binary just as small.

**N2. `panic = "abort"` also applies to `lwb-bench`**
- Location: `Cargo.toml:31-36` (workspace-wide release profile; Cargo doesn't allow per-package `panic`).
- Problem: the prompt allowed abort "for the app binary". `lwb-bench` (0d) will launch other browsers with fresh temp profiles. With abort, a panic skips `Drop`, so guards that kill child browsers or delete temp profiles won't run, which leaves stray processes that skew the next measurement.
- Fix: for 0d, either add `[profile.bench] inherits = "release"`, `panic = "unwind"` and build the harness with `--profile bench`, or make the harness's cleanup not depend on `Drop`. A one-line note in the 0d prompt is enough for now.

**N3. CLA drops the Apache ICLA's reservation-of-rights sentence**
- Location: `docs/CLA.md`, intro and §2; `CONTRIBUTING.md:62` ("You keep the copyright to your work").
- Problem: the Apache ICLA says "Except for the license granted herein … You reserve all right, title, and interest in and to Your Contributions." The adaptation keeps only a softer "does not change your right to use your own Contributions for any other purpose", and the "Changes from the Apache ICLA" list doesn't mention removing it. CONTRIBUTING promises something the CLA doesn't state outright. The ICLA's "Subject to the terms and conditions of this Agreement" lead-in to §2/§3 is also gone.
- Fix: the owner has already approved the text, so this is for the lawyer review SPEC §5.1 already calls for: restore the reservation sentence, or list its removal under "Changes". If the text changes, update the Gist too (S4).

**N4. Bug template points to private vulnerability reporting, which is disabled**
- Location: `.github/ISSUE_TEMPLATE/bug_report.yml:9`.
- Problem: it says to use "Report a vulnerability" on the Security tab, but `GET /repos/…/private-vulnerability-reporting` returns `enabled: false`, so that button doesn't exist. There's also no `SECURITY.md`.
- Fix: owner enables Settings → Code security → Private vulnerability reporting, and optionally adds a short `SECURITY.md`. This matters for a browser more than for most projects.

**N5. `tester-feedback` label doesn't exist**
- Location: `.github/ISSUE_TEMPLATE/tester_feedback.yml:3`.
- Problem: the repo's labels include `bug` and `enhancement` but not `tester-feedback`. GitHub silently skips labels that don't exist, so tester reports arrive unlabelled.
- Fix: owner creates the label, or the 0a follow-up adds it with `gh label create tester-feedback`.

**N6. cargo-deny version floats in CI**
- Location: `.github/workflows/ci.yml:65` (`EmbarkStudios/cargo-deny-action@v2`).
- Problem: `@v2` pulls whatever cargo-deny release the action currently bundles. cargo-deny has made breaking config changes before (the v2 `deny.toml` migration), so an upstream release can turn CI red on an unrelated PR. The PR tested locally with 0.21.0.
- Fix: pin to an exact action release (or SHA) and bump deliberately, matching the toolchain-pin philosophy of DEC-006. Dependabot for `github-actions` would keep both action pins current.

**N7. PR description is stale after the second commit**
- Location: PR #4 body.
- Problem: it still says `docs/CLA.md` is "for owner approval" and lists "Approve `docs/CLA.md`" and "Enable CLA Assistant" as to-dos, while `2b61458` marks both done. The "CI green" box is unchecked, though CI is green. It also doesn't mention DEC-007 or the SPEC §14 edit.
- Fix: update the description, which also helps the orchestrator's final pass. List DEC-007 under deviations, since it changes SPEC §14 (a process change the orchestrator should explicitly confirm).

**N8. Unrecorded extra license on the allow-list**
- Location: `deny.toml:24` (`Apache-2.0 WITH LLVM-exception`).
- Problem: not in the prompt's list or DEC-006. It's strictly more permissive than Apache-2.0 and common in the Rust ecosystem (rustix, wasi crates), so it's harmless, but the allow-list is a policy and each entry should be deliberate.
- Fix: add a one-line comment in `deny.toml` or a bullet in DEC-006.

**N9. CI hardening: cancelled runs on `main`, no job timeouts**
- Location: `.github/workflows/ci.yml:9-11` and jobs.
- Problem: `cancel-in-progress: true` also applies to pushes to `main`, so two quick merges leave the first commit on `main` with a cancelled run and no green result. Jobs have no `timeout-minutes`, so a hung step (e.g. a GUI test on macOS in 0c) burns the 6-hour default.
- Fix: `cancel-in-progress: ${{ github.event_name == 'pull_request' }}`; add `timeout-minutes: 30` (or so) to each job.

## Not findings (checked and fine)
- `lwb-core` purity, `lwb-engine` public API, `// SAFETY:` comments, private-API policy, SPEC §6.6: nothing to check yet. No dependencies, no `unsafe`, no engine code. The `unsafe_op_in_unsafe_fn` deny is in place for later.
- Placeholder tests are trivial, as they should be for stubs. There's no policy logic yet that needs real tests.
- `deny.toml` has no `targets` filter, so Windows- and macOS-only dependencies will be license-checked on the Linux job too. Good.
- `rustup toolchain install` with no arguments correctly reads `rust-toolchain.toml` on all three runners (CI logs), so skipping a third-party toolchain action works.
- The workflow uses `pull_request`, not `pull_request_target`, with `contents: read`. Fork PRs can't reach secrets or write access.
- `.gitignore` ignores `.claude/settings.local.json`. `.claude/worktrees/` is excluded locally via `.git/info/exclude` on the owner's machine, which is enough.
- Branch protection on `main` is not set yet (`404 Branch not protected`). This is already listed as an owner to-do in the PR and STATUS.
