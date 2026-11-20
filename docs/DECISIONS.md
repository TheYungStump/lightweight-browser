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
  - **CLA Assistant has no committed config.** The app is configured in its web UI. Contributors sign a **GitHub Gist** (revision `ca1b2bc`), and `docs/CLA.md` mirrors it, so the two must change together. The owner enables it by hand.
  - **Allow-list extra:** `Apache-2.0 WITH LLVM-exception` is allowed beyond the prompt's list. It's strictly more permissive than Apache-2.0 and common in the Rust ecosystem (e.g. `rustix`, `wasi`).
  - **CI action pins:** `EmbarkStudios/cargo-deny-action` is pinned to the commit of `v2.1.2` (bundles cargo-deny 0.21.0), because a new cargo-deny release can break `deny.toml`. Bump it deliberately, like the toolchain.
  - Our own crates are `publish = false` and licensed `FSL-1.1-ALv2`. `deny.toml` ignores private crates for license checks.
- Alternatives considered: the floating `stable` channel (not reproducible); `dtolnay/rust-toolchain` pinned to a SHA (works, but adds a third-party action for no gain).
- Spec sections affected: §6.1 (MSRV policy refined), §10 Phase 0a.

## DEC-007: Review sessions may be launched by the implementation session  (2026-11-19, phase 0a)
- Context: SPEC §14 says a fresh session reviews each PR, but not who starts it. The desktop app lets one Claude Code session start another, and the owner asked to use that for reviews.
- Decision: once a PR is open and CI is green, the owner may ask the implementation session to start the review as a separate session. It gets **fresh context** (no fork of the implementer's conversation), its **own worktree**, and only the filled-in `docs/prompts/review-pr.md` prompt. The implementer doesn't pass on its own findings or edit the review. Starting the review by hand stays equally valid.
- Alternatives considered: owner-started sessions only (more manual steps); a forked session (would inherit the implementer's reasoning and defeat the clean-context rule).
- Spec sections affected: §14 step 3; `docs/prompts/_kickoff.md` step 4.

## DEC-008: Positioning: "the most memory-efficient browser that never loses your work"  (2026-11-20, spec v0.3)
- Context: the research report (`docs/research/2026-11-browser-positioning.md`) found that "uses less memory" is already sold by Brave, Chrome and Edge. The owner's own test: Brave 8.1 GB with Memory Saver off, 2.6 GB on Maximum. Every shipping tab discarder reloads from the URL and loses typed text and scroll position, and users report it as data loss. The report ranks "memory × never lose your place" first of five angles, with calm/no-AI as tone, transparency as proof, and old/low-RAM PCs as the first-user channel.
- Decision (owner): headline **"The most memory-efficient browser that never loses your work."** Our difference: as lean as Memory Saver on its most aggressive setting, but in-use tabs are never slept and sleeping tabs come back where the user left them. Brand tone is **calm**: no AI unless the user turns it on, no crypto, no shopping features, no news feeds, no telemetry by default. First audiences: heavy-tab users, and people on older or low-RAM PCs (8 GB, Windows 10, which keeps WebView2 updates until at least Oct 2028 per the report). Old PCs are an audience, not the differentiator. Battery, privacy and built-in blocking are supporting claims only. The §4 headline comparison is against **Brave Maximum and Edge at its most aggressive sleeping-tabs setting**, not their defaults.
- Realistic-mix target (owner): ≤ 100% of Brave Maximum and of Edge's most aggressive setting, with all 5 must-stay-alive tabs alive and the half-filled form's contents intact after a forced discard; ≤ 35% of Brave with Memory Saver off stays as a second line; results report how many of the 5 tabs Brave and Edge kept alive. All targets stay provisional until 0d.
- Not put into the spec: figures the report marks as thin or single-source (Windows 10 market share, Zen's user count, Brave Containers, the Edge forum quotes taken from search extracts). Quotes must be spot-checked on the source pages before any public copy.
- Alternatives considered: "no AI" as the headline (the report: it's the loudest complaint, but only ~1% of Firefox users used the kill switch, so it works as tone, not as a switching reason); old hardware as the headline (an audience, and Brave Maximum already serves it); headline comparison against competitor defaults (easier to win, but not credible).
- Spec sections affected: header, §1, §2 (G2, G3, G5, G7), §3, §4, §9, §12.

## DEC-009: Never lose your work: protection first, then capture and restore  (2026-11-20, spec v0.3)
- Context: the report's top finding is that no browser restores a discarded tab's form contents or position, and that users distrust exception lists that fail silently. The spec already protected in-use tabs (§7.3) but restored only the scroll position.
- Decision (owner): two layers, in this order of importance:
  1. **Don't sleep tabs holding work.** The §7.3 protection rules are the primary guarantee, and the public claim rests on them.
  2. **Restore what we can when a tab is slept anyway** (§7.5): capture form field values, the focused field and the scroll position via the isolated-world page script, plus a screenshot; after reload, restore them (best effort, stable selectors) and show the screenshot while the page loads. Never captured: password fields, `autocomplete` `cc-*` / `one-time-code` / `new-password`, hidden fields, file inputs, and anything in private windows. Form data is in memory only and dropped on tab close (surviving a restart is D10). Screenshots live on disk, not in RAM, and are deleted on tab close. The spec states the limits honestly: in-page app state can't be restored by a reload.
- Implementation choices made while writing it (owner-approved in the plan): cross-origin frames are not captured (this also excludes embedded payment forms); captured data is untrusted page-channel input (schema-validated, size-capped, restored only into the same origin; `contenteditable` restored as text or sanitized HTML); form state is captured at freeze time or just before a discard, the screenshot when the user leaves the tab (hidden views may not render); the screenshot cache is cleared at startup. The spec had no existing "screenshots on disk, not in RAM" rule; §7.5 now states it.
- Heavy-tab users (owner): a 500-tab session restore test in 1b acceptance (session saving is now explicitly incremental, §6.8); tab search including sleeping tabs moves to 2b; optional auto-archive (off by default) in 4a. The realistic-mix benchmark adds a forced-discard form check, run against every browser including Edge at its most aggressive setting.
- Alternatives considered: restore only (no layer-1 emphasis), rejected because a reload can't recover in-page state; persisting form data to disk by default, deferred to D10 for privacy.
- Spec sections affected: §2 G3/G7, §4, §6.3, §6.6, §6.8, §7.3, §7.5, §8.4, §9, §10 (1b, 2a, 2b, 4a), §12, §13 D10.

## DEC-010: AI is one slider, off by default; D9 widened  (2026-11-20, spec v0.3)
- Context: the report found forced AI is the loudest browser complaint of 2025–2026, and that it conflicts with the Phase 6 AI toolkit and AI-credits revenue line unless AI stays truly absent by default. It also notes that prompt injection in page-acting agents is unsolved.
- Decision (owner): a **3-position slider, "No AI" / "On request" / "Full AI", default No AI**, on the onboarding page and in Settings. No AI = no AI code loaded, no AI network requests, no AI interface anywhere except the Settings AI section and the onboarding slider. On request = only on an explicit click or shortcut. Full AI adds proactive features. Every AI feature has its own toggle; changing one shows "Custom". **Updates never turn AI on**: a new AI feature defaults on only for Full AI users and is announced in the release notes; this promise is public. Core features (hibernation, protection, blocking, restore) never depend on AI. AI credits stay a revenue line (§5.2). Providers unchanged (BYOK free, or prepaid credits via our proxy). Packaging is D11; the cost model (on-device OS models, BYOK, credits, no bundled local model by default) is D13. A draft feature list is in §8.8; the final list is decided in `docs/AI-SPEC.md` before Phase 6.
- D9 widened (owner): "automation of the browser or pages: outside surfaces and in-browser agents that act (click, type, submit, navigate) on the user's behalf". AI features that only read a page on the user's request are governed by the slider, not D9. The `bench`-feature control channel is a dev-only tool outside D9. §6.7 now says AI features may issue browser commands (e.g. an accepted tab-tidy suggestion) but never act on pages.
- Alternatives considered: dropping AI entirely (loses a revenue line the owner wants to keep); a separate AI build or download only (D11 keeps this open, leaning towards bundling if small).
- Spec sections affected: §2 (G8, non-goals), §5.2, §6.7, §8.8, §10 Phase 6, §13 D9, D11, D13, D14.

## DEC-011: Optional onboarding page  (2026-11-20, spec v0.3)
- Context: the AI slider, the hibernation preset and imports (including stranded Arc users; Arc stops working on 22 December 2026 per the report) need a first-run home, but the calm tone rules out a forced setup.
- Decision (owner): a first-launch page in Phase 4a (§8.9), entirely optional ("Skip, just browse"; sensible defaults everywhere), re-openable from Settings, no account or email. Steps: import (Chrome, Brave, Edge, Firefox, Arc: bookmarks, history, open tabs/sessions; Arc from its local sidebar/spaces data), theme, hibernation preset, ad blocking, a fair search-engine choice list (the default is tied to search-partnership revenue, D4), and the AI slider starting at No AI. Arc spaces import as bookmark folders in 4a, and Phase 5a offers to convert them into spaces (owner). Passwords and cookies are never imported; each importer gets a fuzz target.
- Alternatives considered: no onboarding (settings scattered, AI slider hard to find); a mandatory setup wizard (contradicts the calm tone). The report rated an Arc importer low priority because Arc users are mostly on macOS; the owner chose to include it.
- Spec sections affected: §8.4, §8.9, §10 Phase 4a, §13 D15.

## DEC-012: Spaces with optional separate logins, as Phase 5a  (2026-11-20, spec v0.3)
- Context: the report found Firefox-style containers still unmet on Chromium and buildable with public APIs on all three platforms: WebView2 profiles (which share processes), `WKWebsiteDataStore` identifiers (macOS 14+), and WebKitGTK network sessions.
- Decision (owner): spaces where each space can optionally have separate cookies/logins (§8.10), in a new **Phase 5a**; extensions become **5b** (extensions will need to know about spaces, and 4a is already large). Measure each extra WebView2 profile's memory cost when built. The macOS API needs 14+ while §3 says 13+: open decision D12.
- Alternatives considered: spaces in 4a (too much for one sub-phase); separate WebView2 user-data folders per space (more processes and memory than profiles).
- Spec sections affected: §3, §6.3, §6.4, §8.7, §8.10, §10 Phase 5, §13 D12.

## DEC-013: Spec v0.3 smaller additions: 0b checklist, benchmark, risks, trust page  (2026-11-20, spec v0.3)
- Context: the report flagged claims that depend on unresearched technical facts, and proof points the benchmark lacks.
- Decision (owner):
  - **Phase 0b checklist** adds: what the WebView2 runtime sends to Microsoft and how to turn it off (decides whether "no telemetry by default" is true on Windows); DRM playback (Netflix, Disney+, Spotify web) and which DRM system; whether WebView2 child processes can go in a Job Object or get EcoQoS, for a future resource limiter.
  - **§9 benchmark:** an 8 GB RAM Windows 10 machine where available; optional per-process energy via `powercfg /srumutil`; Edge sleeping-tab modes; the forced-discard form check.
  - **§12 risks:** WebKitGTK on weak Linux hardware; WebView2 support on Windows 10 ending (at least Oct 2028); Kagi dropping Orion for Windows/Linux as a warning about maintaining WebKit off Apple platforms; the headline claim being tested publicly.
  - **Phase 4b:** a public "How we're funded / what we collect" page.
  - **Phase 7:** `docs/MARKET.md` starts from the research report, which is committed with this spec version.
- Spec sections affected: §9, §10 (0b, 0d, 4b, 7), §12.
