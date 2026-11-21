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
- Heavy-tab users (owner): a 500-tab session restore test in 1b acceptance (session saving is now explicitly incremental, §6.8); tab search including sleeping tabs moves to 2c; optional auto-archive (off by default) in 4b (phase ids after the DEC-014 split). The realistic-mix benchmark adds a forced-discard form check, run against every browser including Edge at its most aggressive setting.
- Alternatives considered: restore only (no layer-1 emphasis), rejected because a reload can't recover in-page state; persisting form data to disk by default, deferred to D10 for privacy.
- Spec sections affected: §2 G3/G7, §4, §6.3, §6.6, §6.8, §7.3, §7.5, §8.4, §9, §10 (1b, 2b, 2c, 4b), §12, §13 D10.

## DEC-010: AI is one slider, off by default; D9 widened  (2026-11-20, spec v0.3)
- Context: the report found forced AI is the loudest browser complaint of 2025–2026, and that it conflicts with the Phase 6 AI toolkit and AI-credits revenue line unless AI stays truly absent by default. It also notes that prompt injection in page-acting agents is unsolved.
- Decision (owner): a **3-position slider, "No AI" / "On request" / "Full AI", default No AI**, on the onboarding page and in Settings. No AI = no AI code loaded, no AI network requests, no AI interface anywhere except the Settings AI section and the onboarding slider. On request = only on an explicit click or shortcut. Full AI adds proactive features. Every AI feature has its own toggle; changing one shows "Custom". **Updates never turn AI on**: a new AI feature defaults on only for Full AI users and is announced in the release notes; this promise is public. Core features (hibernation, protection, blocking, restore) never depend on AI. AI credits stay a revenue line (§5.2). Providers unchanged (BYOK free, or prepaid credits via our proxy). Packaging is D11; the cost model (on-device OS models, BYOK, credits, no bundled local model by default) is D13. A draft feature list is in §8.8; the final list is decided in `docs/AI-SPEC.md` before Phase 6.
- D9 widened (owner): "automation of the browser or pages: outside surfaces and in-browser agents that act (click, type, submit, navigate) on the user's behalf". AI features that only read a page on the user's request are governed by the slider, not D9. The `bench`-feature control channel is a dev-only tool outside D9. §6.7 now says AI features may issue browser commands (e.g. an accepted tab-tidy suggestion) but never act on pages.
- Alternatives considered: dropping AI entirely (loses a revenue line the owner wants to keep); a separate AI build or download only (D11 keeps this open, leaning towards bundling if small).
- Spec sections affected: §2 (G8, non-goals), §5.2, §6.7, §8.8, §10 Phase 6, §13 D9, D11, D13, D14.

## DEC-011: Optional onboarding page  (2026-11-20, spec v0.3)
- Context: the AI slider, the hibernation preset and imports (including stranded Arc users; Arc stops working on 22 December 2026 per the report) need a first-run home, but the calm tone rules out a forced setup.
- Decision (owner): a first-launch page in Phase 4c (§8.9; 4a before the DEC-014 split), entirely optional ("Skip, just browse"; sensible defaults everywhere), re-openable from Settings, no account or email. Steps: import (Chrome, Brave, Edge, Firefox, Arc: bookmarks, history, open tabs/sessions; Arc from its local sidebar/spaces data), theme, hibernation preset, ad blocking, a fair search-engine choice list (the default is tied to search-partnership revenue, D4), and the AI slider starting at No AI. Arc spaces import as bookmark folders in 4c, and Phase 5a offers to convert them into spaces (owner). Passwords and cookies are never imported; each importer gets a fuzz target.
- Alternatives considered: no onboarding (settings scattered, AI slider hard to find); a mandatory setup wizard (contradicts the calm tone). The report rated an Arc importer low priority because Arc users are mostly on macOS; the owner chose to include it.
- Spec sections affected: §8.4, §8.9, §10 Phase 4c, §13 D15.

## DEC-012: Spaces with optional separate logins, as Phase 5a  (2026-11-20, spec v0.3)
- Context: the report found Firefox-style containers still unmet on Chromium and buildable with public APIs on all three platforms: WebView2 profiles (which share processes), `WKWebsiteDataStore` identifiers (macOS 14+), and WebKitGTK network sessions.
- Decision (owner): spaces where each space can optionally have separate cookies/logins (§8.10), in a new **Phase 5a**; extensions become **5b** (extensions will need to know about spaces, and Phase 4 is already large). Measure each extra WebView2 profile's memory cost when built. The macOS API needs 14+ while §3 says 13+: open decision D12.
- Alternatives considered: spaces in Phase 4 (too much for one sub-phase); separate WebView2 user-data folders per space (more processes and memory than profiles).
- Spec sections affected: §3, §6.3, §6.4, §8.7, §8.10, §10 Phase 5, §13 D12.

## DEC-013: Spec v0.3 smaller additions: 0b checklist, benchmark, risks, trust page  (2026-11-20, spec v0.3)
- Context: the report flagged claims that depend on unresearched technical facts, and proof points the benchmark lacks.
- Decision (owner):
  - **Phase 0b checklist** adds: what the WebView2 runtime sends to Microsoft and how to turn it off (decides whether "no telemetry by default" is true on Windows); DRM playback (Netflix, Disney+, Spotify web) and which DRM system; whether WebView2 child processes can go in a Job Object or get EcoQoS, for a future resource limiter.
  - **§9 benchmark:** an 8 GB RAM Windows 10 machine where available; optional per-process energy via `powercfg /srumutil`; Edge sleeping-tab modes; the forced-discard form check.
  - **§12 risks:** WebKitGTK on weak Linux hardware; WebView2 support on Windows 10 ending (at least Oct 2028); Kagi dropping Orion for Windows/Linux as a warning about maintaining WebKit off Apple platforms; the headline claim being tested publicly.
  - **Phase 4d** (4b before the DEC-014 split): a public "How we're funded / what we collect" page.
  - **Phase 7:** `docs/MARKET.md` starts from the research report, which is committed with this spec version.
- Spec sections affected: §9, §10 (0b, 0d, 4d, 7), §12.

## DEC-014: Spec v0.3 incorporates review 2  (2026-11-20, spec v0.3)
- Context: `docs/reviews/spec-review-2.md` (approve with changes: 4 blocking, 20 should fix, 10 nice to have). The owner asked for all findings to be folded in.
- Decision: all findings accepted, with these notes:
  - **B1 (Memory Saver already exempts in-use tabs):** §1 now admits the overlap (Chrome's documented exemptions) and names what competitors don't do. The headline (DEC-008) stands as the owner's decision, but is confirmed for public use only after 0d runs the realistic mix and forced-discard check against Brave and Edge, and public wording is scoped to what the benchmark measures. §12's Memory Saver risk is raised to High.
  - **B2–B3 (restore safety, screenshots):** §7.5 now captures only user-edited fields, restores only on matching URL and form fingerprint, never overwrites changed content or fires `change` automatically, never writes into `contenteditable` (a "view / copy" bar instead), and encrypts screenshots with a per-run in-memory key in the local cache folder, skipping sensitive pages. This **supersedes** DEC-009's "sanitized HTML" restore and its "screenshot when the user leaves the tab" timing (now: debounced while visible).
  - **B4:** "alive" = not discarded and not frozen, JS running (heartbeat). The chat tab is protected by each browser's own keep-awake list (rule 5), using a site on no default list, rather than a new page-reported rule. The form is filled with OS-level input in every browser.
  - **S11 (targets):** the owner's real-world target becomes "≤ Brave Maximum in the same session"; **the ≤ 35% second line is kept** because the owner asked for it on 2026-11-20, with a note that it's looser than Brave Maximum's measured ~32% and is recalibrated in 0d. Safari becomes a required macOS benchmark config.
  - **S4:** one WebView2 environment; private windows and spaces are controller options; the engine trait takes an opaque `ProfileId` from 1a.
  - **S8:** wording on WebView2's required diagnostic data written now; crash dumps kept local via `IsCustomCrashReportingEnabled`; new open decision **D16** (SmartScreen default).
  - **S17–S19 (AI):** AI output reaches a page only after the user previews and confirms; "loaded" means a separately loaded module or helper process; the public promise is worded "updates never raise your AI level, and new AI features stay off unless you chose Full AI" (new features off for Custom); provider retention is disclosed.
  - **S20 (phase splits):** 1b → 1b (storage + session) and 1c (engine-event UIs); old 1c → 1d. 2a → 2a (lifecycle, signals, protection) and 2b (capture and restore); old 2b → 2c. 4a → 4a (browsing data, windows), 4b (polish, default browser + single instance), 4c (onboarding + importers); old 4b → 4d. Phase ids in DEC-009 to DEC-013 are updated to the new numbering.
  - Also added: web notifications (1c; macOS shim), passkeys on macOS and Linux DRM as risks and 0c checks, single-instance URL handoff, popup inheritance, WebKitGTK's sender-frame gap, the `PROCESS_MEMORY_COUNTERS_EX2` fallback, drag and drop, an autoplay setting (3b), the Arc importer as best effort, D12's ephemeral-space option, and D13 facts to check. Claims marked "(from memory)" by the reviewer are marked (verify) in the spec.
- Alternatives considered: deferring the phase splits until each phase prompt is written (rejected: the splits change acceptance criteria now).
- Spec sections affected: §1, §2 G3, §4, §6.3, §6.4, §6.6, §7.2, §7.3, §7.5, §7.6, §8.4, §8.6, §8.8–§8.10, §9, §10 (0b–0d, 1a–1d, 2a–2c, 3b, 4a–4d, 5a, 6), §12, §13 (D2, D8, D10, D12, D13, D16).

## DEC-015: Windows page scripts run as a bundled extension's content scripts  (2026-11-21, phase 0b)
- Context: SPEC §6.6 asked 0b to confirm that WebView2 host scripts run in the page's main world, and to evaluate a bundled unpacked extension instead. The spike confirmed both (`docs/spikes/phase0b-windows.md` items 19, 23). Host scripts (`AddScriptToExecuteOnDocumentCreated`, `ExecuteScript`) share the page's globals, and the page intercepted our `chrome.webview.postMessage`. A bundled MV3 extension's content scripts run in an isolated world but have **no `chrome.webview`**. They can reach the host with a simple `no-cors` POST to a reserved host (`https://lwb-channel.invalid/…`), which our `WebResourceRequested` handler answers itself (no network), with a per-run token in the body. A page forging that request without the token is rejected. The extension **doesn't run in private (InPrivate) views**, even though `AddBrowserExtension` succeeds on the private profile.
- Recommendation (for orchestrator/owner approval):
  - Isolated-world page scripts on Windows (form/media detection, form capture and restore, scroll restore, volume boost, cosmetic filtering, forced dark) ship as a bundled extension. It is loaded into every non-private profile at startup (and every space's profile in 5a), with `AreBrowserExtensionsEnabled` set once on the single environment (§6.4).
  - Scripts that must be in the main world (`getUserMedia`/`AudioContext` wrappers, `beforeunload` detection) stay host scripts, as §6.6 already says.
  - **Private windows:** no isolated world, so isolated-world features either run as main-world host scripts there (their signals are page-reported and forgeable anyway) or are off. §7.5 already captures nothing in private windows. Volume boost, cosmetic filtering and forced dark would need the main-world fallback.
  - The channel token is regenerated every run and written into the extension folder in our app-data dir. Messages stay untrusted, schema-validated input (§6.6) whatever the channel.
  - To verify in 1a/2a: whether a page's service worker can see the content script's channel request (expected no, not tested), and the channel's latency under load.
- Alternatives considered: main-world host scripts only (the page can see, patch, forge or stall them, as §6.6 assumed); `postMessage` from the content script via the DOM (the page can read it).
- Spec sections affected: §6.3 (`EngineCaps.isolated_world` true on Windows except private windows), §6.6, §7.2, §7.5, §8.1–§8.3.

## DEC-016: `MemoryUsageTargetLevel = Low` trims memory, it doesn't free it; report commit too  (2026-11-21, phase 0b)
- Context: in the spike, a hidden Wikipedia renderer with `MemoryUsageTargetLevel = Low` dropped from 108 MB to 7 MB **private working set**, but its **private commit** only fell from 122 MB to 112 MB. The pages went to the pagefile/standby list and fault back in when the tab is used (visible again: 80 MB ws). `TrySuspend` freed ~8 MB of commit. SPEC §7.6 makes private working set our metric (what Task Manager shows), which rewards trimming.
- Recommendation: keep private working set as the user-facing and headline metric, because it is what users compare and Chromium browsers trim the same way. But `lwb-memprobe` and `lwb-bench` also record **private commit** for every process and publish it alongside, and public copy never presents trimming as freed memory. Keep `Low` for Background tabs (cheap, and it lets Windows reclaim RAM under pressure). The §7.4 budget planner should decide on private working set (what the user set the limit against), with commit shown on `lwb://memory`. The orchestrator should confirm the metric choice when 0d recalibrates §4.
- Spec sections affected: §4 (statistic definitions), §7.1 (Background → `Low`), §7.4, §7.6, §9.

## DEC-017: We create the single WebView2 environment ourselves; SmartScreen via the documented API  (2026-11-21, phase 0b)
- Context: wry creates its own environment per view unless given one, and its default browser arguments are `--disable-features=msWebOOUI,msPdfOOUI,msSmartScreenProtection`, which **silently turn off SmartScreen** through an undocumented flag. wry also has no option for `IsCustomCrashReportingEnabled`.
- Decision (spike-verified): the Windows backend calls `CreateCoreWebView2EnvironmentWithOptions` itself (via `webview2-com`) with our options (`IsCustomCrashReportingEnabled = true`, `AreBrowserExtensionsEnabled` per DEC-015, our browser arguments) and passes the result to every view, popups included, with wry's `with_environment`. This is exactly SPEC §6.4's "exactly one environment". SmartScreen (D16) is controlled with the documented `ICoreWebView2Settings8::IsReputationCheckingRequired`, not the undocumented flag. Both removed the SmartScreen connections in the spike. If D16 chooses "on", we pass no SmartScreen flag. The setting applies per user-data folder, and Windows' own "SmartScreen for Microsoft Edge" switch can force it off.
- Also: `IsCustomCrashReportingEnabled = true` verifiably stops Crashpad's upload attempts (0 vs 1 attempt per report in Crashpad's database). Dumps are still written to `<profile>\EBWebView\Crashpad\reports` (`FailureReportFolderPath`), 6–9 MB each, so 4d needs a size cap or cleanup.
- Spec sections affected: §6.4, §8.6, §13 D16.

## DEC-018: Engine caveats from the Windows spike  (2026-11-21, phase 0b)
- Context: smaller findings from `docs/spikes/phase0b-windows.md` that the backend and policy layer must handle.
- Decision / guidance for 1a and 2a:
  1. **`TrySuspend` only works on hidden views** (`IsVisible = false`); on a visible view it fails synchronously with `0x8007139F`. Freeze only after hiding.
  2. **No engine-initiated resume event exists.** `ExecuteScript` on a suspended view runs and returns **without resuming it** (timers stay paused), so §7.5 can capture state while frozen. The backend must synthesize `Resumed{by_engine}` itself, by checking `IsSuspended` after navigation, visibility changes or script calls.
  3. **Register request filters before the first navigation.** A view built with a URL starts navigating before we can add the `WebResourceRequested` filter, so its first document request (and its POST flag) is missed. Build with no URL, register handlers, then navigate.
  4. **Shared-worker requests are delivered to every view in the profile**, not just the one that started the worker. The blocker evaluates each delivery; per-tab counters must not assume one view per request.
  5. **HTTP Basic auth** first completes the navigation with status 17 (credentials required), then retries after we answer. The UI must not show that first completion as an error.
  6. **Resource limiter (future):** putting our process in a Job Object before creating the environment puts the whole WebView2 tree in it, and a whole-browser CPU hard cap works. Kill-on-close works, but WebView2 also exits on its own within 4 s when the host dies. **Chromium manages EcoQoS on its own processes** (background renderers are already throttled) and overwrites ours on visibility changes, so per-renderer EcoQoS from us isn't viable. Per-renderer job caps weren't tried.
  7. **Test automation:** DevTools-protocol input counts as user activation for permissions and popups but not for fullscreen; tests needing fullscreen use OS input (`SendInput`).
  8. **`IsDocumentPlayingAudio` flickers during continuous playback.** On Spotify, Netflix, Disney+ and YouTube it drops to false for ~0.08–0.25 s and back, every few seconds to minutes, with no audible gap (it's Chromium's audibility detection, so quiet passages count). Debounce before acting on it: treat a view as playing until it has reported false for ~2 s. This matters for the tab audio indicator and for §7.2 rule 1 (never freeze a tab playing audio), which must not freeze in one of those dips.
  9. **Popups replace the opener on screen in the spike** (it adopts them as tabs), so the owner didn't notice the Google sign-in ran in a popup. Phase 1 should make popups visibly a new tab (or a small window) and return focus to the opener when they close.
- Spec sections affected: §6.3 (event list: `Resumed{by_engine}` is synthesized on Windows), §7.1, §7.2, §7.5, §8.1, §12 (resource limiter note).

## DEC-019: WebView2 network traffic with every app control off; follow-up in a sandbox  (2026-11-21, phase 0b)
- Context: SPEC §10 0b asked what the WebView2 runtime sends to Microsoft and which controls affect it. Owner constraint: built-in tools only, no root certificate on the owner's machine; pair observations with Microsoft's documentation; if something is still contacted with every control off, record the hosts and recommend a sandbox/VM follow-up rather than doing it now.
- Findings (`docs/spikes/phase0b-windows.md`, "Telemetry"): Microsoft documents that required diagnostic data is collected regardless of settings and that apps control only crash reporting and SmartScreen. In non-admin TCP observation, SmartScreen's hosts disappear with `IsReputationCheckingRequired = false` (or wry's flag). **With every app-level control on, the runtime still contacts `config.edge.skype.com` (Experimentation & Configuration Service) at startup and `edge.microsoft.com` about 60 s after start.** No `*.events.data.microsoft.com` connection came from our process tree in a 20-minute idle run. Required data may travel through Windows' own diagnostics service, which isn't attributable.
- Recommendation:
  1. Keep §1's wording. 4d's "what we collect" page lists `config.edge.skype.com` and `edge.microsoft.com` as runtime-originated, pending the follow-up.
  2. **Follow-up (before 4d):** inspect payloads in Windows Sandbox or a VM with a TLS-intercepting proxy (never on the owner's main machine). Questions: what these two hosts receive, whether the `ExperimentationAndConfigurationServiceControl = 0` policy stops the first (A5 tests reachability only), and whether anything leaves via Windows' diagnostics service when only our app runs.
  3. D16: SmartScreen sends the visited URL's information on each navigation when on. Decide the default with that fact. DEC-017 covers the mechanism.
- Spec sections affected: §1 (brand tone wording), §10 Phase 4d, §13 D16.
