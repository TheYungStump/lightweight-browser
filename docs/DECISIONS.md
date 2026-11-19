# Decision log

Record every decision that changes or refines `docs/SPEC.md`: an API that didn't exist, a chosen workaround, a resolved open decision from SPEC §13. Newest at the bottom.

Template:

```
## DEC-NNN: <short title>  (YYYY-MM-DD, phase N)
- Context: what forced the decision
- Decision: what we're doing
- Alternatives considered:
- Spec sections affected:
```

## DEC-001: License is FSL-1.1-ALv2 with a CLA  (2026-11-18, spec)
- Context: the code should be public, but nobody should be able to sell a competing copy.
- Decision: Functional Source License 1.1 with Apache-2.0 as the future license (each release becomes Apache-2.0 after two years). Contributors sign a CLA via CLA Assistant.
- Alternatives considered: GPL/MPL (allow commercial forks), PolyForm Shield (never becomes open source).
- Spec sections affected: §5.1, §13 D1.

## DEC-002: Repository is public  (2026-11-18, spec)
- Context: free GitHub Actions minutes on standard runners (macOS minutes cost 10x on private repos); trust and marketing.
- Decision: the repo is public. The LICENSE file lands in Phase 0a.
- Spec sections affected: §13 D3.

## DEC-003: Spec v0.2 incorporates review 1  (2026-11-18, spec)
- Context: `docs/reviews/spec-review-1.md` (approve with changes).
- Decision: all blocking findings (B1–B4) and should-fix items accepted, with these adjustments: the MV3-extension blocker is evaluated in 3a, not Phase 0; early cheap Windows signing is a conditional decision (D8) rather than a plan change; private Apple APIs are allowed under a policy (D7). Phases split into sub-phases 0a–0d, 1a–1c, 2a–2b, 3a–3b, 4a–4b.
- Spec sections affected: most; see the git diff of `docs/SPEC.md`.

## DEC-004: Private Apple APIs allowed behind a feature flag  (2026-11-18, spec)
- Context: per-tab mute and per-tab memory attribution on macOS appear to need private WebKit APIs. The owner wants to keep a future Mac App Store build (and a mobile app) possible; the App Store rejects private API use, while direct downloads with notarization don't.
- Decision: private APIs are allowed only behind the `macos-spi` Cargo feature, enabled for direct-download builds and disabled for any App Store build. Each use needs a runtime `respondsToSelector:` check and a public fallback, so the app works with the feature off.
- Alternatives considered: no private APIs at all (loses per-tab mute and per-tab memory on macOS); unrestricted use (blocks the App Store later).
- Spec sections affected: §2 non-goals, §6.6, §13 D7.

## DEC-005: Lessons carried over from the owner's previous project  (2026-11-19, spec)
- Context: the owner's screenwriting app spec worked well. Reviewed it for practices that fit this project.
- Decision: added (1) a command layer: every action is a named command through one dispatcher (§6.7, Phase 1a); (2) installer-size, cold-start and input-latency targets, a stated statistic for every target, and CI-enforced budgets where CI can measure reliably (§4, 0d, 4b); (3) fuzz targets for every untrusted-input parser (§6.6); (4) a scriptlet-bundle license check (§8.1); (5) a note that free open-source signing programmes likely exclude FSL (§11); (6) a sourced market/pain-point doc in Phase 7; (7) D9: no external automation surface without a separate decision.
- Not carried over: clean-room process (we aren't recreating a specific product), GPL licensing (DEC-001), pay-what-you-want/paid store builds (doesn't fit a browser's revenue model).
- Spec sections affected: §4, §6.6, §6.7 (new), §6.8 (renumbered storage), §8.1, §10, §11, §13.

## DEC-006: Phase 0a toolchain, CI and license-check choices  (2026-11-18, phase 0a)
- Context: Phase 0a set up the workspace, tooling and CI. A few details weren't fixed by the spec or the prompt.
- Decision (owner-approved where noted):
  - **Toolchain pinned exactly** to `1.100.0` in `rust-toolchain.toml`, with `rust-version = "1.100.0"` to match (owner). Bumps are deliberate: at the start of a phase, in their own commit. This refines §6.1 "MSRV = latest stable at the start of each phase".
  - **LGPL isn't on the cargo-deny allow-list** (owner). cargo-deny checks Rust crates only, never system libraries such as WebKitGTK, so "LGPL only for system libraries" needs no allow-list entry. A crate that needs LGPL gets a named `exceptions` entry with a reason.
  - **No third-party toolchain action in CI.** Runners ship rustup, and `rustup toolchain install` reads `rust-toolchain.toml`, so the toolchain is pinned by our own file. The remaining actions are first-party or widely used ones pinned to a major version (`actions/checkout@v7`, `Swatinem/rust-cache@v2`, `EmbarkStudios/cargo-deny-action@v2`).
  - **Linux CI packages:** `libwebkit2gtk-4.1-dev libgtk-3-dev libsoup-3.0-dev`. wry's README and its own CI install only `libwebkit2gtk-4.1-dev`; tao and muda need `libgtk-3-dev`. Nothing else is required.
  - **CLA Assistant has no committed config.** The app is configured in its web UI and points at `docs/CLA.md`. The owner enables it by hand.
  - Our own crates are `publish = false` and licensed `FSL-1.1-ALv2`. `deny.toml` ignores private crates for license checks.
- Alternatives considered: the floating `stable` channel (not reproducible); `dtolnay/rust-toolchain` pinned to a SHA (works, but adds a third-party action for no gain).
- Spec sections affected: §6.1 (MSRV policy refined), §10 Phase 0a.

## DEC-007: Review sessions may be launched by the implementation session  (2026-11-19, phase 0a)
- Context: SPEC §14 says a fresh session reviews each PR, but not who starts it. The desktop app lets one Claude Code session start another, and the owner asked to use that for reviews.
- Decision: once a PR is open and CI is green, the owner may ask the implementation session to start the review as a separate session. It gets **fresh context** (no fork of the implementer's conversation), its **own worktree**, and only the filled-in `docs/prompts/review-pr.md` prompt. The implementer doesn't pass on its own findings or edit the review. Starting the review by hand stays equally valid.
- Alternatives considered: owner-started sessions only (more manual steps); a forked session (would inherit the implementer's reasoning and defeat the clean-context rule).
- Spec sections affected: §14 step 3; `docs/prompts/_kickoff.md` step 4.
