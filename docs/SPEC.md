# Lightweight Browser — Project Specification

> **Status:** DRAFT v0.1 — awaiting review and approval.
> **Working name:** "Lightweight Browser" (crate prefix `lwb`). The final name is an open decision (§13). Renaming is cheap until Phase 4.
> **Owner:** project owner (human). **Orchestrator:** cloud Claude session. **Implementers:** local Claude Code sessions, one or more per phase.

This document is the single source of truth. Implementation sessions must read it before writing code, and must not silently diverge from it. If reality forces a change (an API doesn't exist, a design doesn't work), record it in `docs/DECISIONS.md` and flag it in the phase PR.

---

## 1. Vision

A fast, private, cross-platform desktop browser whose defining feature is **using dramatically less RAM than Chrome, Brave or Edge for the same set of tabs**, without making the user babysit their tabs.

It achieves this by:
1. Using the operating system's own web engine instead of bundling a second copy of Chromium.
2. Managing tabs aggressively but *intelligently*: background tabs are frozen and then unloaded, while tabs the user is actually using (a podcast playing in the background, a video call, a half-filled form) are never touched.
3. Building in the features people normally install memory-hungry extensions for: ad/tracker blocking, dark mode, volume boost.
4. Showing the user, honestly, where their memory is going.

Primary audience: people with many tabs open on 8–16 GB machines who are frustrated by browser memory use. The owner's own baseline: **Brave at ~6.5 GB** with their usual tabs.

## 2. Goals and non-goals

### Goals
- G1. Daily-driver-quality browsing on Windows, macOS and Linux desktop.
- G2. Measurably lower memory than Brave/Chrome on a reproducible benchmark (§9), with published results.
- G3. Smart tab hibernation and a user-set memory budget (similar in spirit to Opera GX's RAM limiter, but protecting tabs that are in use).
- G4. Built-in ad/tracker blocking, dark mode, per-tab mute and volume boost.
- G5. Privacy by default: no telemetry unless the user opts in.
- G6. An architecture that lets us swap or add web engines later (Servo, or our own) without rewriting the browser.
- G7. Later: an extension system, then an optional AI toolkit that costs **zero memory when unused**.

### Non-goals (for now)
- Writing our own HTML/CSS/JS engine. Revisit only after Phase 6, as a separate decision (§10, Phase 8).
- Mobile (iOS/Android).
- Full Chrome Web Store compatibility on all platforms (see §8.6).
- Built-in password manager or sync (users keep their existing password manager; sync is a later paid-tier candidate).
- Code signing before we have positive tester feedback (§11).

## 3. Platforms

| Platform | Minimum | Engine | Priority |
|---|---|---|---|
| Windows | Windows 10 22H2, Windows 11 (x64; ARM64 later) | Microsoft Edge WebView2 (Evergreen runtime) | **Primary**: owner's main machine |
| macOS | macOS 13 Ventura+ (Apple Silicon; Intel best-effort) | WKWebView | High |
| Linux | Ubuntu 22.04 / 24.04 desktop | WebKitGTK (webkit2gtk-4.1) | Medium |

Note: the owner's Ubuntu machine is a server. Linux support targets *desktop* Linux. The server can host backend services later (update manifest, feedback intake, AI credit proxy; see §10).

## 4. Success metrics

All numbers are measured with the benchmark in §9, on the same machine, against fresh-profile Brave.

| Metric | Target |
|---|---|
| Empty window (one new-tab page), total memory | ≤ 200 MB on Windows, ≤ 150 MB on macOS |
| Browser shell's own process (our Rust code + UI) | ≤ 60 MB |
| Standard 20-tab set, **hibernation off** | ≤ 85% of Brave (from the shared OS engine and built-in blocking alone) |
| Standard 20-tab set, **default hibernation on**, after 30 min idle with 1 audio tab playing | ≤ 35% of Brave |
| Owner's real-world usage (Brave ~6.5 GB) | ≤ 2 GB |
| Restoring a hibernated tab (time to first paint, typical news site, cached) | ≤ 1.5 s |
| Wrongly hibernated tabs (a tab hibernated while playing audio, in a call, or holding unsaved input) | **0**. Treat any occurrence as a bug |

## 5. Licensing and business model

### 5.1 License (decision needed, see §13)
Goal: the code is public (free CI, trust, contributors, marketing), but nobody can take it and sell a competing browser.

- GPL/MPL (open source) does **not** meet this goal. Anyone may fork and sell, as long as they publish their source.
- **Recommended: Functional Source License, `FSL-1.1-ALv2`.** Anyone can read, build, modify and use the code for anything **except** offering a competing commercial product. Each release automatically becomes Apache-2.0 two years after publication, which earns community goodwill and avoids "abandonware" fears.
- Alternative if the code should never become open: **PolyForm Shield 1.0**, the same non-compete idea with no conversion.
- Either way: add a **Contributor License Agreement** (e.g. the CLA Assistant GitHub app) so we keep the right to relicense or dual-license, and treat the **product name as a trademark**. The license does not protect the name; registering it later does.
- Third-party licenses we depend on are compatible with this: wry/tao (MIT/Apache-2.0), adblock-rust (MPL-2.0, file-level copyleft: changes to *its* files must be published), WebKitGTK (LGPL, dynamically linked system library), WebView2 SDK (Microsoft's distributable runtime terms). Keep a `THIRD_PARTY_LICENSES` file generated by `cargo-about` or `cargo-deny`.
- Not legal advice. Get a lawyer to look over it before anything is commercialised.

### 5.2 Revenue (owner's preference, in order)
1. **Search partnerships.** A default-search royalty or affiliate deal. Needs real user numbers, so it comes after Phase 4. The user can always change the engine, and we never sell browsing data.
2. **Donations and sponsorships.** GitHub Sponsors, Open Collective, Ko-fi links on the About page and the website from Phase 4.
3. **AI credits** (Phase 6). Prepaid credits for the AI toolkit, served through our own proxy. "Bring your own API key" stays free.
4. Possibly later: a paid tier (sync, premium themes, priority support).

Hard rules for every revenue feature: no ads injected into pages, no selling user data, no telemetry without opt-in.

## 6. Architecture

### 6.1 Tech stack
- **Language:** Rust, stable, edition 2024. MSRV = latest stable at the start of each phase.
- **Windowing / event loop:** `tao`.
- **Webviews:** `wry` (multiple child webviews per window), plus **direct platform bindings where wry doesn't expose what we need**:
  - Windows: `webview2-com` / `windows` crates, reached via wry's `WebViewExtWindows` (controller → `ICoreWebView2*`).
  - macOS: `objc2` / `objc2-web-kit`, reached via wry's `WebViewExtMacOS` (→ `WKWebView`).
  - Linux: `webkit2gtk` crate, reached via wry's `WebViewExtUnix` (→ `WebKitWebView`).
- **Browser UI (the "chrome": tab strip, address bar, menus):** HTML/CSS/vanilla JS in a dedicated small webview, embedded with `include_dir`/`rust-embed`. No front-end framework and no bundler in v0.x. Phase 0 measures this webview's memory cost. If it exceeds ~40 MB over baseline, evaluate a native alternative (egui) before Phase 1.
- **Storage:** SQLite via `rusqlite` (bundled feature). Settings as TOML or SQLite (implementer's choice, documented).
- **Ad blocking:** `adblock` (Brave's adblock-rust).
- **Async/runtime:** avoid a full tokio runtime in the shell unless needed. Prefer the tao event loop plus a small thread pool. (Every MB counts.)
- **Logging:** `tracing` with a rolling file appender. No network logging.
- **Packaging:** `cargo-packager` (MSI/NSIS, .app/.dmg, .deb/AppImage).

### 6.2 Workspace layout
```
/Cargo.toml                 # workspace
/crates/lwb-app/            # binary: window, event loop, wiring everything together
/crates/lwb-core/           # platform-independent logic, NO engine/OS deps:
                            #   tab model, lifecycle state machine, hibernation policy,
                            #   memory-budget algorithm, session model, settings model
/crates/lwb-engine/         # Engine + EngineView traits and per-OS backends
    src/windows/            #   WebView2 backend
    src/macos/              #   WKWebView backend
    src/linux/              #   WebKitGTK backend
/crates/lwb-memprobe/       # per-OS process memory measurement (also used by the bench tool)
/crates/lwb-blocker/        # adblock-rust wrapper, filter-list download/update/compile
/crates/lwb-storage/        # SQLite: sessions, history, bookmarks, settings
/ui/                        # browser chrome HTML/CSS/JS + internal pages (lwb://...)
/tools/lwb-bench/           # memory benchmark harness (§9)
/docs/                      # SPEC.md, DECISIONS.md, STATUS.md, spikes/, testing/, prompts/
```
Rule: **`lwb-core` must be fully unit-testable with no OS or engine.** All policy decisions, like "should this tab hibernate?" or "which tab do we unload to meet the budget?", live there as pure functions or state machines with an injected clock.

### 6.3 Engine abstraction
Everything above `lwb-engine` talks to traits, never to wry or platform types directly. Sketch (the implementer refines it):

```rust
pub trait Engine {
    type View: EngineView;
    fn create_view(&self, parent: &Window, opts: ViewOptions) -> Result<Self::View>;
    fn capabilities(&self) -> EngineCaps;           // what this backend supports
    fn process_snapshot(&self) -> Vec<ProcessInfo>; // for memory attribution
    fn set_content_blocker(&self, rules: CompiledRules) -> Result<()>;
}

pub trait EngineView {
    fn navigate(&self, url: &Url);
    fn reload(&self); fn stop(&self); fn go_back(&self); fn go_forward(&self);
    fn set_bounds(&self, r: Rect); fn set_visible(&self, v: bool);
    fn set_zoom(&self, z: f64);
    fn set_muted(&self, m: bool);
    fn eval_js(&self, script: &str);
    fn suspend(&self) -> SuspendResult; // Tier-2 freeze; may be Unsupported
    fn resume(&self);
    fn set_memory_target(&self, low: bool); // WebView2 MemoryUsageTargetLevel; no-op elsewhere
    fn is_playing_audio(&self) -> Option<bool>;
    fn capture_state(&self) -> CaptureState; // mic / camera / screen in use
    fn snapshot(&self, cb: Box<dyn FnOnce(Option<Image>)>);
}

pub struct EngineCaps { pub suspend: bool, pub audio_state: bool, pub capture_state: bool,
                        pub per_view_process_attribution: bool, pub network_intercept: bool,
                        pub content_rule_lists: bool, pub chrome_extensions: bool }
```
Events flow out of views as an `EngineEvent` enum (`NavigationStarted`, `TitleChanged`, `FaviconChanged`, `AudioStateChanged`, `NewWindowRequested`, `DownloadStarted`, `ProcessCrashed`, `PageMessage(...)`, ...) delivered to the app's event loop.

Every backend-specific feature is reported through `EngineCaps`, and the policy layer degrades gracefully (e.g. no Tier-2 freeze on WebKit → go straight from Throttled to Discarded).

### 6.4 Process model
- **Windows:** all tabs are created from **one shared `CoreWebView2Environment`**, so they share the browser, GPU, network and utility processes. Renderers are per-site (Chromium site isolation). Experiment flag (off by default, benchmark-only at first): `AdditionalBrowserArguments` such as `--renderer-process-limit=N`, to measure the memory/safety trade-off.
- **macOS:** WebKit manages WebContent processes itself (a shared process pool is the default on modern macOS). We control only views.
- **Linux:** WebKitGTK multi-process. Investigate sharing a `WebKitWebContext` and `related-view` for same-origin popups.

### 6.5 Browser UI webview and internal pages
- One chrome webview per window, docked at the top. Content webviews are child views below it; only the active tab's view is visible.
- Internal pages use a custom protocol `lwb://` (wry custom protocol): `lwb://newtab`, `lwb://settings`, `lwb://memory` (memory dashboard), `lwb://history`, `lwb://bookmarks`, `lwb://about`, `lwb://hibernated` (placeholder shown for a discarded tab).

### 6.6 IPC and security model
- **Only the chrome webview and `lwb://` pages** get an IPC channel to Rust. Content webviews get **no** privileged IPC.
- Some features need injected scripts in content pages (dirty-form detection, media detection fallback, scroll capture, volume boost, cosmetic filtering). These scripts talk back over a **separate, narrow message channel**. Every message is treated as untrusted input from the web: schema-validated, rate-limited, and mapped only to that tab's own state (a page can never affect other tabs or settings).
- Injected scripts run in an isolated world where the engine supports it (WebKit: `WKContentWorld`; WebView2: verify in Phase 0, and if no isolated world is available, use frozen closures and never expose functions or message ports on `window`). Document the approach per backend.
- `lwb://` pages must not be navigable or iframe-able from web content.
- Allowed schemes in the address bar: `http`, `https`, `lwb`, `file` (with a warning), `about:blank`. Others prompt or are refused.
- Downloads go through engine download hooks into a user-chosen folder. Never auto-open downloads.
- `NewWindowRequested`: open as a new tab, **except** popups that need `window.opener` (OAuth "Sign in with Google/Microsoft/Apple"). These must keep the opener relationship (WebView2: assign the new view to `NewWindowRequestedEventArgs.NewWindow`; WebKit: return the new view from `createWebViewWith`). **Phase 0 must verify this works through wry or our direct bindings**, because many logins break without it.

### 6.7 Storage
SQLite file in the per-OS app data dir (`directories` crate). Tables (v1): `sessions`, `tabs` (incl. per-tab back/forward stack and scroll position), `history`, `bookmarks`, `site_settings` (zoom, dark mode, keep-awake, blocker exceptions), `settings`. Engine profile data (cookies, cache) stays in the engine's own data store, pointed at our app data dir. Migrations are versioned (`PRAGMA user_version`).

## 7. Memory management (core feature)

### 7.1 Tab lifecycle
```
           (user switches away)          (idle ≥ T_freeze)          (idle ≥ T_discard, or budget pressure)
 Active ───────────────────────► Background ───────────────────► Frozen ──────────────────────────────► Discarded
   ▲            (throttled by engine, view hidden,                (engine suspend; JS paused,          (view destroyed; we keep URL,
   │             memory-target=low on WebView2)                    memory partly reclaimable)           title, favicon, history stack,
   │                                                                                                    scroll pos, optional thumbnail)
   └──────────────────────────── (user activates tab: resume or reload) ◄───────────────────────────────────┘
```
- **Frozen** exists only where `EngineCaps.suspend` is true (WebView2 `TrySuspend`). Elsewhere Background goes straight to Discarded.
- **Defaults** (user-configurable, with presets "Relaxed / Balanced / Aggressive"): `T_freeze = 5 min`, `T_discard = 30 min` (Balanced).
- **Lazy session restore:** on startup, every restored tab except the active one starts **Discarded**. This alone is a big win over mainstream browsers.
- Background-opened tabs (middle-click) load normally, since the user expects them ready, but become eligible for hibernation like any other tab.

### 7.2 Activity signals (per tab)
Gathered from native APIs first, injected-script fallback second. Each backend documents which signals it provides.

| Signal | Windows (WebView2) | macOS (WKWebView) | Linux (WebKitGTK) | JS fallback |
|---|---|---|---|---|
| Playing audio | `IsDocumentPlayingAudio` + changed event | `requestMediaPlaybackState` (macOS 12+) — verify | `is-playing-audio` property | `play`/`pause`/`ended` on media elements; AudioContext state |
| Mic / camera / screen capture | permission-requested + capture state APIs — verify | `microphoneCaptureState`, `cameraCaptureState` | permission requests + `is-capturing-*` — verify | `getUserMedia` wrapper |
| Unsaved form input | — | — | — | `input` events on editable fields not yet submitted; `beforeunload` handler present |
| Picture-in-picture / fullscreen | events | — verify | — verify | `enterpictureinpicture`, `fullscreenchange` |
| Active download | download events | download delegate | download signals | — |
| Last user interaction time | focus/visibility from our UI | same | same | — |

"verify" = Phase 0 spike must confirm the API exists and behaves as expected.

### 7.3 Protection rules (a protected tab is never frozen or discarded)
A background tab is **protected** if any of these hold:
1. It is playing audio, or played audio in the last **5 minutes** (grace period for pausing a podcast).
2. It is capturing mic, camera or screen (calls, meetings).
3. It has unsaved form input, or registered a `beforeunload` handler after user interaction.
4. It is in picture-in-picture or fullscreen.
5. It has an active download or upload.
6. The user marked it **Keep awake** (tab context menu and an icon on the tab).
7. Its site is on the user's **Always keep awake** list (e.g. `music.youtube.com`, `discord.com`, `teams.microsoft.com`). Ship a small editable default list.
8. It is the active tab, or the **previously active** tab (fast Alt-Tab-style switching), for the first 2 minutes after leaving it.
9. It is the chrome webview or an `lwb://` page with in-progress state (settings being edited).

Protection is re-evaluated on every signal change and on every policy tick.

### 7.4 Memory budget (RAM limiter)
- User setting: **Memory limit** (off / auto / custom). "Auto" = 30% of physical RAM, minimum 1.5 GB. Custom is a slider.
- **Policy tick** every 5 s (also triggered immediately by OS memory-pressure signals: Windows `CreateMemoryResourceNotification`, macOS `DISPATCH_SOURCE_TYPE_MEMORYPRESSURE`, Linux PSI `/proc/pressure/memory`).
- On each tick:
  1. Measure total browser memory (§7.6).
  2. Apply idle timers (Background→Frozen→Discarded) to eligible, unprotected tabs.
  3. If total > limit: choose the eligible, unprotected tab with the **oldest last-active time** (tie-break: largest attributed memory), freeze it if possible, otherwise discard it. Then wait a **10 s cooldown** for memory to actually be released before the next forced action.
  4. If total > limit and every candidate is protected: **do nothing destructive**. Show a non-intrusive indicator ("Over memory limit: 3 protected tabs") that links to `lwb://memory`.
- All of this is a pure function in `lwb-core`: `fn plan(state: &BrowserState, now: Instant, mem: MemSample, cfg: &Policy) -> Vec<TabAction>`. It is fully unit-tested, including the podcast scenario: a YouTube tab playing audio in the background is never selected, even under heavy budget pressure.

### 7.5 Restore
- **Frozen → Active:** `resume()`, instant.
- **Discarded → Active:** create a view, navigate to the saved URL, restore scroll position via an injected script after load (best effort), and rebuild back/forward from our stored per-tab history stack. Back/Forward on a restored tab navigates through our stack, since the engine's own history was lost.
- Discarded tabs show their saved title and favicon in the tab strip, with a subtle "sleeping" style. Optional (Phase 2 stretch): a thumbnail captured before discard, stored **on disk**, not in RAM.

### 7.6 Measuring memory (`lwb-memprobe`)
- **Metric per OS** (chosen to match what users see in their task manager):
  - Windows: **private working set** of each process.
  - macOS: **`phys_footprint`** (what Activity Monitor's "Memory" column shows) via `proc_pid_rusage`.
  - Linux: **PSS** from `/proc/<pid>/smaps_rollup`.
- **Which processes belong to us:**
  - Windows: our process + WebView2 processes from `CoreWebView2Environment.GetProcessExtendedInfos` (also gives per-renderer frame→tab mapping).
  - macOS: WebKit's WebContent/Networking/GPU processes are XPC services, **not children** of our process. Identify via the responsible-PID relationship or process enumeration; Phase 0 picks a reliable method.
  - Linux: our process tree (WebKitGTK subprocesses are children).
- **Per-tab attribution:** exact-ish on Windows (renderer→frames mapping; a shared renderer's memory is split and marked "shared"). On macOS/Linux, v0.x shows the **total plus engine-process breakdown**; per-tab numbers are labelled "estimate" or omitted until a reliable method exists.

## 8. Built-in features

### 8.1 Ad and tracker blocking (`lwb-blocker`)
- Engine: adblock-rust. Default lists: EasyList + EasyPrivacy (plus uBlock Origin's base filters if license-compatible; check). Lists are **downloaded and updated at runtime** (weekly, with ETag), not committed to the repo, and cached compiled on disk.
- **Windows:** network-level blocking via WebView2 `WebResourceRequested` (filter on all resource types), with the adblock engine deciding per request. Measure the overhead in benchmarks.
- **macOS and Linux:** convert the rules to WebKit **content rule list** JSON (adblock-rust's content-blocking conversion) → `WKContentRuleListStore` / `WebKitUserContentFilterStore`. Respect WebKit's per-list rule limit by splitting into several lists.
- Cosmetic filtering (hiding ad elements): inject the generated CSS per page.
- Per-site toggle in the address bar ("Shields"-style), stored in `site_settings`.
- Don't promise YouTube ad blocking. It is an arms race, best effort only.

### 8.2 Dark mode
1. **Native preference:** tell pages the user prefers dark (`prefers-color-scheme: dark`). WebView2 `PreferredColorScheme`; WebKit follows the app/window appearance.
2. **Forced dark** for sites without a dark theme: injected CSS filter mode (invert + hue-rotate, re-inverting images and video), per-site toggle, off by default. A smarter "dynamic" mode (like Dark Reader's) is a later enhancement.
3. Browser UI itself: light, dark, or follow system.

### 8.3 Audio controls
- Per-tab **mute** via native APIs (WebView2 `IsMuted`; WebKit media-muted APIs), with a speaker icon on audible tabs (click to mute).
- Per-tab **volume boost** (0–400%) via injected Web Audio: route media elements through a `GainNode`. Known limitation: cross-origin media without CORS can't be routed (it would go silent). Detect this and fall back to 100% with a tooltip explaining why.

### 8.4 Navigation and everyday features (Phase 1/4)
Address bar with search (default engine configurable; ship with privacy-respecting options plus the common ones), autocomplete from history/bookmarks, back/forward/reload/stop, zoom (per-site, remembered), find in page, keyboard shortcuts (Ctrl/Cmd + T, W, Shift+T, L, R, F, Tab/Shift+Tab, 1–9, +/−/0, Alt+←/→), tab drag-reorder, pinned tabs, downloads list, history, bookmarks, bookmark import (Chrome/Brave/Edge `Bookmarks` JSON; Firefox `places.sqlite` later), crashed-tab page with Reload.

### 8.5 Updates
Phase 4: on startup (at most daily), check the GitHub Releases API and **notify only**. Releases include an **ed25519 signature** (minisign) of each artifact, checked before telling the user an update is genuine. This works without paid OS code signing. Self-installing updates come later.

### 8.6 Extensions (Phase 5)
1. **Built-in features first** (§8.1–8.3).
2. **Our own lightweight extension format:** manifest + content scripts + CSS + a small, permissioned API (storage, tab info, messaging). It deliberately mirrors a subset of WebExtensions (`content_scripts`, `storage`) so simple Chrome extensions can be ported easily. Works on all three OSes.
3. **Windows only, experimental:** load unpacked/CRX Chrome extensions via WebView2's browser-extension API (`AreBrowserExtensionsEnabled` + `AddBrowserExtension`). Compatibility is partial. Password managers are the hard case. Note: Chrome-extension features would then differ by OS, so mark them clearly.

### 8.7 AI toolkit (Phase 6)
- Optional and **off by default**. When off, it loads nothing and costs zero memory (verified by benchmark).
- Initial features: summarize page, ask questions about the page, "tidy my tabs" (group/close suggestions), explain a selection.
- Providers: bring-your-own API key (free), or **prepaid credits** through our proxy (hosted on the owner's server). Page content is sent only on explicit user action and never stored server-side beyond the request.
- Details are specified in a separate `docs/AI-SPEC.md` before Phase 6 starts.

## 9. Benchmark methodology (`tools/lwb-bench`)
- Launches a browser with a **fresh profile** and a fixed URL list, waits for the page loads to settle, then samples memory (§7.6 metrics, whole process set) every 5 s for 5 min. Reports median and p90. 3 runs per configuration; report the median of the medians.
- **Browsers:** ours (hibernation off; hibernation on with default settings + 30-min idle simulation), Brave, Chrome, Edge (Windows), Safari (macOS, manual/limited), Firefox. Fresh profiles, no extensions, default settings, built-in ad blocking left on where the browser ships it (Brave) and noted.
- **Standard tab set (`bench/tabs-standard.txt`), 20 logged-out pages:** a YouTube video (playing, for the audio-protection scenario), Wikipedia article, two news sites, Reddit thread, GitHub repo page, Stack Overflow question, an MDN page, a Google Maps location, a shopping product page, a recipe site (ad-heavy), Twitch channel page, a docs site, a blog, a weather site, a sports site, a long article site, an image-heavy site, a web app demo (e.g. Excalidraw), and a search results page. The exact URLs are fixed in Phase 0 and versioned.
- **Owner scenario:** a manual checklist that reproduces the owner's usual Brave session (logged in), measured in both browsers.
- Output: CSV + markdown summary into `bench/results/<date>-<os>-<machine>.md`. These results become the public marketing claims, so the methodology must be documented and reproducible.

## 10. Phases

Each phase = one feature branch (`phase-N-<slug>`) → PR to `main`. It is done when **all acceptance criteria are met, CI is green, the manual test checklist (`docs/testing/phase-N.md`) has been run on Windows (and macOS when available), and the review session approves it.**

### Phase 0: Foundation and engine spike
**Goal:** prove the risky assumptions before building on them.
- Repo setup: workspace skeleton (§6.2), `rustfmt`, `clippy` (`-D warnings`), `cargo-deny` (licenses/advisories), license file + CLA (after §13 decision), `.editorconfig`, issue templates.
- CI: GitHub Actions matrix (windows-latest, macos-latest, ubuntu-latest): build, test, clippy, fmt. Requires a **public repo** (free standard runners). Fallback if private: self-hosted runners on the owner's own machines.
- **Engine spike app** (throwaway quality is fine, in `spikes/engine-spike/`): one window, chrome webview + 2 content webviews switchable by buttons, on all 3 OSes. Verify and record in `docs/spikes/phase0-engine.md` a ✅/❌ table for each item:
  - multiple child webviews + `set_bounds`/visibility in one window via wry;
  - access to the native webview object on each OS (for the APIs below);
  - WebView2: `TrySuspend`/`Resume`, `MemoryUsageTargetLevel`, `IsDocumentPlayingAudio`, `IsMuted`, `GetProcessExtendedInfos`, `WebResourceRequested`, `NewWindowRequested` with opener preserved (test a real Google sign-in popup);
  - WKWebView: media playback state, capture state, content rule lists, `createWebViewWith` opener popups, process identification for memory;
  - WebKitGTK: `is-playing-audio`, mute, user content filters, related views for popups;
  - memory cost of the chrome webview (empty window total, with and without it);
  - custom `lwb://` protocol serving embedded files.
- **Benchmark harness v0:** `lwb-memprobe` + `lwb-bench` measuring Brave, Chrome and Edge on Windows with the standard tab set. **Produces the baseline numbers we're trying to beat.**
- **Exit:** the spike report plus a go/no-go per platform. Any ❌ becomes a decision in `docs/DECISIONS.md` (workaround, direct bindings, or scope change) before Phase 1 starts.

### Phase 1: Core browser
- Real app built on the `Engine` traits (not the spike).
- Window with chrome webview: tab strip (new, close, switch, reorder, pinned), address bar (URL or search), back/forward/reload/stop, loading indicator, title + favicon.
- `lwb://newtab` (minimal: search box + a few shortcuts), `lwb://about`.
- New-window requests → new tab (opener preserved for popups per §6.6).
- Downloads: save to Downloads folder, simple download bar/list.
- Keyboard shortcuts (§8.4 subset), per-tab zoom.
- Settings storage + `lwb://settings` (search engine, startup behaviour, theme).
- Session save/restore with **lazy restore** (non-active tabs start discarded, §7.1). This needs the Discarded state from §7 in a minimal form.
- Crashed-renderer page with Reload.
- **Acceptance:** the owner can browse normally on Windows for a day (YouTube, Gmail, GitHub, banking site, Google sign-in popup) with no blocking bugs. Empty-window memory measured and recorded. Builds and launches on macOS and Linux in CI; smoke-tested manually on macOS when the MacBook arrives.

### Phase 2: Memory management
- Full lifecycle (§7.1) with Frozen on Windows.
- Activity signals (§7.2) with native + JS-fallback sources per backend.
- All protection rules (§7.3), Keep-awake per tab, Always-keep-awake site list.
- Memory budget (§7.4) with presets and OS memory-pressure triggers.
- Restore with scroll and history stack (§7.5).
- `lwb://memory` dashboard: total, per-process breakdown, per-tab (where supported), each tab's state, a "hibernate now" button, and an explanation of why a tab is protected.
- Tab-strip indicators: sleeping, frozen, protected (with reason in the tooltip), audio.
- Unit tests for `plan()` covering: idle timers, budget pressure, every protection rule, the podcast scenario, the all-protected case, cooldown.
- **Acceptance:** §4 memory targets for "hibernation on" met or the gap explained; zero wrongly hibernated tabs in a scripted test (audio tab, form tab, `getUserMedia` tab left in background for 60 min with the "Aggressive" preset); first public benchmark write-up.

### Phase 3: Built-in features
- Ad/tracker blocking (§8.1), with per-site toggle and a blocked-count badge.
- Dark mode (§8.2).
- Audio: mute + volume boost (§8.3).
- Find in page.
- **Acceptance:** blocking works on all 3 OSes against a test page of known ad/tracker URLs; benchmark rerun showing blocking's memory effect; volume boost works on YouTube.

### Phase 4: Daily-driver polish and tester release
- History (+ `lwb://history`), bookmarks (+ bar, `lwb://bookmarks`), bookmark import from Chrome/Brave/Edge.
- Address-bar autocomplete.
- Final name and icon (if decided), About page with donation/sponsor links.
- Packaging with `cargo-packager`: Windows installer (NSIS/MSI, unsigned), macOS `.dmg` (ad-hoc signed `.app`, not notarized), Linux `.deb` + AppImage.
- `docs/TESTERS.md`: how to install unsigned builds (Windows SmartScreen → "More info" → "Run anyway"; macOS → System Settings → Privacy & Security → "Open Anyway", or `xattr -dr com.apple.quarantine`), how to report bugs, and what data (none) we collect.
- Update notifier with minisign verification (§8.5).
- Opt-in crash reports (local minidump + "send report" button; no automatic upload).
- Feedback: GitHub issue templates + an in-app "Send feedback" link.
- **Acceptance:** an unsigned build installs on clean Windows 11 and macOS machines by following `TESTERS.md`; first 5–10 external testers onboarded.

### Phase 5: Extensions
Per §8.6: our extension format + API, a small first-party extension gallery (ports of popular simple extensions), and the experimental Windows Chrome-extension loader.

### Phase 6: AI toolkit and credits
Per §8.7, after writing `docs/AI-SPEC.md`. Includes the credit proxy on the owner's server and a payment provider integration.

### Phase 7: Growth and revenue
Website, search-partnership outreach, sponsorships, code signing (Windows certificate, Apple Developer ID + notarization) once tester feedback is positive, auto-update, optional sync (paid-tier candidate).

### Phase 8: Engine research (decision gate)
Evaluate adding **Servo** as an alternative backend behind the `Engine` trait (Rust-native, embeddable), and decide whether owning an engine is worth it. Output: a research report and recommendation, not a commitment.

## 11. Distribution before code signing
- Builds are unsigned (Windows) or ad-hoc signed (macOS: Apple Silicon requires at least ad-hoc signatures, and Rust's linker applies one to binaries; the `.app` bundle needs `codesign --force --deep -s -`).
- Testers must accept OS warnings. Document this clearly and honestly; publish checksums and minisign signatures for every artifact so testers can verify builds came from us.
- Revisit signing at the start of Phase 7 (or earlier if tester feedback is strong).

## 12. Risks

| Risk | Impact | Mitigation |
|---|---|---|
| wry doesn't expose multi-view or platform hooks we need | High | Phase 0 spike; fall back to direct `webview2-com`/`objc2`/`webkit2gtk` bindings behind our `Engine` trait |
| OAuth popups break (no `window.opener`) | High (logins fail) | Explicit Phase 0 test; native new-window handling |
| WebView2 itself is Chromium, so per-tab cost is similar to Chrome | Medium | Gains come from shared processes, lazy restore, hibernation and blocking; measured honestly from Phase 0 |
| macOS memory attribution for XPC WebKit processes | Medium | Phase 0 picks a method; show totals only if per-tab is unreliable |
| Engines behave differently per OS | Medium | Per-OS manual test checklists; `EngineCaps` gating; Windows is primary |
| Hibernating something the user needed | High (trust) | Conservative protection rules, grace periods, Keep-awake, zero-tolerance metric (§4) |
| Unsigned builds scare testers | Medium | `TESTERS.md`, checksums, signatures; sign in Phase 7 |
| Brave/Chrome add similar memory features | Medium | Compete on combined lightness + transparency (memory dashboard) + cross-platform built-ins |
| Ad-blocking arms race (YouTube) | Low/Medium | Best effort, no promises |
| License misunderstanding | Medium | FSL + CLA, lawyer review before commercialisation |

## 13. Open decisions (owner)
| # | Decision | Recommendation | Status |
|---|---|---|---|
| D1 | License | FSL-1.1-ALv2 (+ CLA) | **Open** |
| D2 | Product name | TBD; check trademark/domain availability | Open (needed by Phase 4) |
| D3 | Repo visibility | Public (free CI, needed for marketing anyway) | Open |
| D4 | Default search engine | Configurable; pick the default when a partnership exists | Open (Phase 4) |
| D5 | Chrome UI tech | HTML chrome webview unless Phase 0 shows > ~40 MB cost | Pending Phase 0 data |
| D6 | Default hibernation preset | Balanced (5 min freeze / 30 min discard) | Open |

## 14. Workflow (orchestration)
1. **Spec review:** a fresh session reviews this spec (prompt in `docs/prompts/review-spec.md`) and produces findings. The owner and orchestrator resolve them, and the owner approves → status becomes `APPROVED v1.0`.
2. **Per phase:** the owner starts a local implementation session with `docs/prompts/phase-N.md` (the orchestrator writes each one when the previous phase is merged). The implementer works on `phase-N-<slug>`, keeps `docs/STATUS.md` updated, records deviations in `docs/DECISIONS.md`, and opens a PR.
3. **Review:** a fresh review session checks the PR against this spec's acceptance criteria and §6.6 security rules. The orchestrator does a final review; the owner merges.
4. Large phases may be split into several sessions. The implementer should stop and write a handoff note in `docs/STATUS.md` rather than work with an overflowing context.
