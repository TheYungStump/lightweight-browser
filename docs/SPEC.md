# Lightweight Browser — Project Specification

> **Status:** DRAFT v0.2 — review 1 (`docs/reviews/spec-review-1.md`) incorporated; awaiting owner approval.
> **Working name:** "Lightweight Browser" (crate prefix `lwb`). The final name is an open decision (§13). Renaming is cheap until Phase 4.
> **Owner:** project owner (human). **Orchestrator:** cloud Claude session. **Implementers:** local Claude Code sessions, one per sub-phase.

This document is the single source of truth. Implementation sessions must read it before writing code, and must not silently diverge from it. If reality forces a change (an API doesn't exist, a design doesn't work), record it in `docs/DECISIONS.md` and flag it in the phase PR.

Items marked **(verify)** are believed correct but must be confirmed in the Phase 0 spikes before anything is built on them.

---

## 1. Vision

A fast, private, cross-platform desktop browser whose defining feature is **using dramatically less RAM than Chrome, Brave or Edge for real-world browsing**, without making the user babysit their tabs.

It achieves this by:
1. Using the operating system's own web engine instead of bundling another copy of Chromium. This is a real gain on macOS and Linux; on Windows, WebView2 is itself Chromium, so the gains there come from points 2–3.
2. Managing tabs aggressively but *intelligently*: lazy session restore, then freezing and unloading background tabs, while tabs the user is actually using (a podcast playing in the background, a video call, a half-filled form) are never touched. Brave and Chrome already discard tabs ("Memory Saver"). We compete by being **smarter about what not to unload, and transparent about why**.
3. Building in what people otherwise install extensions for: ad/tracker blocking, dark mode, volume boost.
4. Showing the user, honestly, where their memory is going.

Primary audience: people with many tabs open on 8–16 GB machines who are frustrated by browser memory use. The owner's own baseline: **Brave at ~8.1 GB (64 processes, Memory Saver off)** with their usual tabs, measured in Windows Task Manager on 2026-11-19.

## 2. Goals and non-goals

### Goals
- G1. Daily-driver-quality browsing on Windows, macOS and Linux desktop.
- G2. Measurably lower memory than Brave/Chrome on a reproducible benchmark (§9), with published results.
- G3. Smart tab hibernation and a user-set memory budget that protects tabs that are in use.
- G4. Built-in ad/tracker blocking, dark mode, per-tab mute and volume boost.
- G5. Privacy by default: no telemetry unless the user opts in; private windows; Global Privacy Control.
- G6. An architecture that lets us swap or add web engines later (e.g. Servo) without rewriting the browser.
- G7. Later: an extension system, then an optional AI toolkit that costs **zero memory when unused**.

### Non-goals (for now)
- Writing our own HTML/CSS/JS engine. Phase 8 is the decision gate for that.
- Mobile (iOS/Android). Keeping `lwb-core` pure Rust keeps it reusable if a mobile app happens later. iOS requires WebKit (outside the EU/Japan exemptions), which our engine abstraction already supports.
- Full Chrome Web Store compatibility on all platforms (see §8.7).
- Built-in password manager or sync (users keep their existing password manager; sync is a later paid-tier candidate).
- Paid code signing before positive tester feedback (§11).

## 3. Platforms

| Platform | Minimum | Engine | Priority |
|---|---|---|---|
| Windows | Windows 10 22H2, Windows 11 (x64; ARM64 later) | Microsoft Edge WebView2 (Evergreen runtime) | **Primary**: owner's main machine |
| macOS | macOS 13 Ventura+ (Apple Silicon; Intel best-effort) | WKWebView | High (no hands-on testing until the owner's MacBook arrives) |
| Linux | Ubuntu 22.04 / 24.04 desktop, **Wayland and X11** | WebKitGTK (webkit2gtk-4.1), via wry's GTK path | Medium |

The owner's Ubuntu machine is a server. Linux support targets *desktop* Linux. The server can host backend services later (update manifest, feedback intake, AI credit proxy).

## 4. Success metrics

> All targets are **provisional until Phase 0d** produces real baselines on the owner's machine. Then the orchestrator recalibrates them and records the result in `DECISIONS.md`.

Measured with the benchmark in §9, same machine, fresh profiles.

| Metric | Target |
|---|---|
| Empty window (one new-tab page), total memory | ≤ 200 MB Windows, ≤ 150 MB macOS |
| Our shell process (Rust code incl. resident ad-block engine) | ≤ 60 MB |
| Standard 20-tab set, hibernation **off**, **Windows** | ≤ 100% of Brave, ≤ 85% of Chrome |
| Standard 20-tab set, hibernation **off**, **macOS** | ≤ 85% of Brave |
| **Realistic mix** (§9: 20 tabs, 5 of which must stay alive), 30 min, default preset | ≤ 60% of Brave **with Memory Saver on**; ≤ 35% of Brave with it off |
| Owner's real-world usage (Brave ~8.1 GB, Memory Saver off) | ≤ 2 GB |
| Restoring a discarded tab (time to first paint, typical news site, cached) | ≤ 1.5 s |
| Page-load time cost of our ad blocker (Windows, median over the standard set) | ≤ +5% vs blocker off |
| Wrongly hibernated tabs (a tab hibernated while protected per §7.3 by a **trusted** signal) | **0**. Treat any occurrence as a bug |

## 5. Licensing and business model

### 5.1 License (decided: DEC-001)
- **`FSL-1.1-ALv2` (Functional Source License, Apache-2.0 future license).** Anyone can read, build, modify and use the code for anything **except** offering a competing commercial product. Each release becomes Apache-2.0 two years after publication.
- **Contributor License Agreement** via the CLA Assistant GitHub app, so the owner keeps the right to relicense or dual-license.
- The product **name is protected by trademark**, not by the license. Check availability when choosing it (D2) and register it before commercialising.
- Third-party licenses are compatible: wry/tao/muda (MIT/Apache-2.0), adblock-rust (MPL-2.0, file-level copyleft: changes to *its* files must be published), WebKitGTK (LGPL, dynamically linked system library), WebView2 runtime (Microsoft redistributable terms). `cargo-deny` enforces an allow-list, and `cargo-about` generates `THIRD_PARTY_LICENSES`.
- **Filter lists** (EasyList/EasyPrivacy: GPLv3 / CC BY-SA 3.0+; uBlock Origin lists: GPLv3) are **downloaded at runtime and never bundled**, so their licenses don't touch our code.
- Not legal advice. Get a lawyer to look over it before anything is commercialised.

### 5.2 Revenue (owner's preference, in order)
1. **Search partnerships.** A default-search royalty or affiliate deal. Needs real user numbers, so it comes after Phase 4. The user can always change the engine, and we never sell browsing data.
2. **Donations and sponsorships.** GitHub Sponsors, Open Collective, Ko-fi links on the About page and the website from Phase 4.
3. **AI credits** (Phase 6). Prepaid credits for the AI toolkit, served through our own proxy. "Bring your own API key" stays free.
4. Possibly later: a paid tier (sync, premium themes, priority support).

Hard rules: no ads injected into pages, no selling user data, no telemetry without opt-in.

## 6. Architecture

### 6.1 Tech stack
- **Language:** Rust, stable, edition 2024. MSRV = latest stable at the start of each phase.
- **Windowing / event loop:** `tao`. **Native menus** (app menu, context menus, tab menu): `muda`.
- **Webviews:** `wry` (multiple child webviews per window), plus **direct platform bindings where wry doesn't expose what we need**:
  - Windows: `webview2-com` / `windows` crates, reached via wry's `WebViewExtWindows` (controller → `ICoreWebView2*`).
  - macOS: `objc2` / `objc2-web-kit`, reached via wry's `WebViewExtMacOS` (→ `WKWebView`).
  - Linux: `webkit2gtk` crate. Child webviews are built with **wry's GTK path** (`build_gtk` into a `gtk::Fixed`/`gtk::Overlay`), not `build_as_child` with a raw window handle, which is X11-only **(verify)**.
- **Browser UI ("chrome": tab strip, address bar):** HTML/CSS/vanilla JS in a dedicated webview, embedded with `rust-embed`. No front-end framework and no bundler in v0.x. See §6.5 for overlays.
- **Storage:** SQLite via `rusqlite` (bundled), WAL mode, for **everything, including settings**.
- **Ad blocking:** `adblock` (Brave's adblock-rust).
- **Async:** no full tokio runtime in the shell unless a DECISIONS entry justifies it. Prefer the tao event loop plus a small thread pool.
- **Logging:** `tracing` with a rolling file appender. No network logging.
- **Packaging:** `cargo-packager` (NSIS/MSI, .app/.dmg, .deb/AppImage).

### 6.2 Workspace layout
```
/Cargo.toml                 # workspace
/crates/lwb-app/            # binary: window, event loop, wiring everything together
/crates/lwb-core/           # platform-independent logic, NO engine/OS deps:
                            #   tab model, lifecycle state machine, hibernation policy,
                            #   memory-budget planner, session model, settings model
/crates/lwb-engine/         # Engine + EngineView traits and per-OS backends
    src/windows/            #   WebView2 backend
    src/macos/              #   WKWebView backend
    src/linux/              #   WebKitGTK backend
/crates/lwb-memprobe/       # per-OS process memory measurement (also used by the bench tool)
/crates/lwb-blocker/        # adblock-rust wrapper, filter-list download/update/compile
/crates/lwb-storage/        # SQLite: sessions, history, bookmarks, site settings, settings
/ui/                        # browser chrome HTML/CSS/JS + internal pages (lwb://...)
/tools/lwb-bench/           # memory benchmark harness (§9)
/spikes/                    # throwaway Phase 0 spike apps (not part of the product build)
/bench/                     # tab sets, results
/docs/                      # SPEC.md, DECISIONS.md, STATUS.md, spikes/, testing/, prompts/, reviews/
```
Rules:
- **`lwb-core` must be fully unit-testable with no OS or engine.** All policy decisions live there as pure functions or state machines with an injected clock.
- **`lwb-engine`'s public API must not expose wry or platform types.**

### 6.3 Engine abstraction
Everything above `lwb-engine` talks to traits. Sketch (the implementer refines it and documents the final form):

```rust
pub trait Engine {
    type View: EngineView;
    fn create_view(&self, parent: &WindowHandle, kind: ViewKind, opts: ViewOptions) -> Result<Self::View>;
    fn capabilities(&self) -> EngineCaps;
    fn process_snapshot(&self) -> Vec<ProcessInfo>;   // for memory attribution
    fn set_content_blocker(&self, rules: CompiledRules) -> Result<()>;
    fn clear_browsing_data(&self, what: DataKinds, profile: ProfileKind) -> Result<()>;
}

pub enum ViewKind { Chrome, Internal, Web }           // see §6.6 privilege rules
pub enum ProfileKind { Normal, Private }              // private = ephemeral data store
pub enum LifecycleHint { Active, Background, Frozen } // each backend maps this to its APIs

pub trait EngineView {
    fn navigate(&self, url: &Url);
    fn reload(&self); fn stop(&self); fn go_back(&self); fn go_forward(&self);
    fn set_bounds(&self, r: PhysicalRect); fn set_visible(&self, v: bool);
    fn set_zoom(&self, z: f64);
    fn set_muted(&self, m: bool);                     // may be a JS fallback; see caps
    fn set_lifecycle_hint(&self, h: LifecycleHint);   // result reported via events
    fn find(&self, q: &FindQuery); fn print(&self); fn open_devtools(&self);
    fn eval_js(&self, script: &str, world: ScriptWorld);
}

pub struct EngineCaps { pub freeze: bool, pub native_audio_state: bool, pub native_mute: bool,
                        pub native_capture_state: bool, pub network_intercept: bool,
                        pub content_rule_lists: bool, pub isolated_world: bool }
```
Events flow out as an `EngineEvent` enum, including at least: `NavigationStarting{url, is_post}`, `NavigationCompleted`, `TitleChanged`, `FaviconChanged`, `AudioStateChanged`, `Frozen`, `Resumed{by_engine}`, `FreezeFailed`, `NewWindowRequested`, `ContextMenuRequested`, `PermissionRequested`, `FullscreenChanged`, `CertificateError`, `AuthRequested`, `JsDialog`, `AcceleratorKey`, `DownloadStarted/Progress/Done`, `ProcessCrashed`, `PageMessage{source_origin, frame, payload}`.

Every backend-specific feature is reported through `EngineCaps`, and the policy layer degrades gracefully (no freeze on WebKit → go straight from Background to Discarded).

`snapshot()` (tab thumbnails) and Chrome-extension capabilities are added when first needed (Phase 2 stretch / Phase 5), not before.

### 6.4 Process model
- **Windows:** create **exactly one `CoreWebView2Environment` per profile kind** (normal, private) and pass it to every view. Views share the browser process only if the user-data folder *and* environment options match exactly. One mismatched `AdditionalBrowserArguments` means a second browser process or a creation error. Renderers are per-site (Chromium site isolation). Experiment flag (bench-only, off by default): `--renderer-process-limit=N`.
- **macOS:** WebKit manages WebContent processes itself. We control only views and data stores.
- **Linux:** WebKitGTK multi-process. Share one `WebKitWebContext` per profile; use `related-view` for popups that need an opener.

### 6.5 Browser UI, overlays and internal pages
- One **chrome view** per window, docked at the top. Content views are native child views below it; only the active tab's view is visible.
- **Overlay problem:** HTML in the chrome view can't draw over content views. Policy:
  1. **Native menus (`muda`)** for the app menu, tab context menu and page context menu.
  2. **Dropdowns/popovers** (address-bar autocomplete, download bubble, permission prompts, find bar, memory-limit indicator): **temporarily grow the chrome view** over the content area with a transparent background while the popover is open, then shrink it back.
  3. If (2) fails a spike test on any OS, fall back to a single shared transparent **overlay view** per window (measure its memory) or, on Windows, WebView2 visual hosting. Record the choice in DECISIONS.
- **Internal pages** (`lwb://newtab`, `settings`, `memory`, `history`, `bookmarks`, `downloads`, `about`) are served by a custom protocol. On Windows, wry exposes custom protocols as `http(s)://lwb.localhost/` **(verify)**; treat that host as internal everywhere.
- **Discarded tabs have no view.** The chrome draws their title, favicon and "sleeping" styling in the tab strip. When a discarded tab is activated, it is normally just reloaded (§7.5). The "click to reload" placeholder (§7.5) is an `Internal` view of the active tab, created only for the active tab.

### 6.6 Security model
**View privilege (fixes the internal/web boundary):**
- Each view has a fixed `ViewKind`, set at creation:
  - `Chrome`: our UI. Full IPC. It can only navigate to its own embedded page; every other navigation is cancelled.
  - `Internal`: `lwb://` pages. Custom protocol registered, IPC restricted to that page's API.
  - `Web`: web content. **No custom protocol registered, no privileged IPC.** Requests to `lwb.localhost` are blocked.
- **Crossing the internal/web boundary swaps the view.** If a tab on `lwb://newtab` navigates to `https://...`, we create a `Web` view for the tab, navigate it, and destroy the internal view, and vice versa. A view's kind never changes.
- **Second layer:** every IPC message is checked against the sender's origin and frame (available on all three backends, **(verify)**). Messages from the wrong origin or from subframes are dropped and logged.

**Injected scripts and page-reported signals:**
- Features that need scripts in web pages (form/media detection fallbacks, scroll restore, volume boost, cosmetic filtering, forced dark mode) use a narrow **page channel**, separate from privileged IPC.
- Scripts run in an **isolated world** where available (`WKContentWorld`, WebKitGTK script worlds). Some fallbacks need the **main world**, where the page can tamper with them: wrapping `getUserMedia`, observing `AudioContext`, detecting `beforeunload` handlers. On WebView2, isolated-world support for host scripts must be verified in Phase 0b. Assume any page script can call `chrome.webview.postMessage`.
- Therefore **every page-channel message is untrusted and forgeable.** It is schema-validated, rate-limited, scoped to its own tab, and can only produce *page-reported* signals (§7.2), which have weaker protection power than trusted ones (§7.3).

**Filter lists:** list URLs are pinned in code and HTTPS-only. List contents are untrusted input. **Scriptlet / redirect resources come only from a pinned resource bundle shipped with the app**, never from downloaded lists, so a hijacked list can't run JS on every site.

**Other rules:**
- Allowed address-bar schemes: `http`, `https`, `lwb` (internal), `file` (with a warning), `about:blank`. Others prompt or are refused.
- Permission requests (camera, mic, location, notifications, clipboard, …) always go through **our** prompt, never the engine default (WebKitGTK denies by default when unhandled; WebView2 shows its own UI). Decisions are stored per site in `site_settings`.
- Certificate errors → our interstitial with "go back" as the default. Never auto-proceed.
- Downloads go to a user-chosen folder and are never auto-opened. Make sure Mark-of-the-Web (Windows) / `com.apple.quarantine` (macOS) is set on downloaded files, setting it ourselves if the engine doesn't.
- `NewWindowRequested`: open as a new tab, **except** popups that need `window.opener` (OAuth "Sign in with Google/Microsoft/Apple"). These keep the opener relationship (WebView2: assign `NewWindowRequestedEventArgs.NewWindow`; WebKit: return the new view from `createWebViewWith` / `create` signal). Phase 0b/0c must verify this works with a real Google sign-in.
- The benchmark control channel (§9) is compiled only with a `bench` Cargo feature that release builds never enable, and is never reachable from web content.
- Private Apple APIs (SPI) are allowed only with a DECISIONS entry, behind a runtime `respondsToSelector:` check with a public fallback, and only inside code gated by the **`macos-spi` Cargo feature** (on for direct-download builds, off for any future Mac App Store build) (D7, DEC-004).

### 6.7 Storage
SQLite file in the per-OS app data dir (`directories` crate), WAL mode. Tables (v1): `sessions`, `tabs` (incl. per-tab back/forward stack, scroll position, last-navigation-was-POST flag), `history`, `bookmarks`, `site_settings` (zoom, dark mode, keep-awake, blocker exceptions, permissions), `settings`. Session state is **saved on every change (debounced ~1 s)**, not only on exit, so crashes lose nothing. Engine profile data (cookies, cache) stays in the engine's own data store, pointed at our app data dir. **Private windows** use an ephemeral engine data store, and none of their tabs or history is written to SQLite. Migrations are versioned (`PRAGMA user_version`).

## 7. Memory management (core feature)

### 7.1 Tab lifecycle
```
           (user switches away)         (idle ≥ T_freeze)          (idle ≥ T_discard, or budget pressure)
 Active ───────────────────────► Background ───────────────────► Frozen ──────────────────────────────► Discarded
   ▲            (engine throttles hidden view;                   (engine suspend; JS paused;          (view destroyed; we keep URL, title,
   │             WebView2 memory target = low)                    memory only partly released)         favicon, history stack, scroll pos)
   └──────────────────────────── (user activates tab: resume or reload) ◄───────────────────────────────────┘
```
- **Frozen** exists only where `EngineCaps.freeze` is true (WebView2 `TrySuspend`). Elsewhere Background goes straight to Discarded. **Freezing is used only by the idle timer**; budget pressure always discards (§7.4).
- The engine can leave Frozen by itself (WebView2 resumes on script execution, navigation or visibility) and `TrySuspend` can fail. The state machine handles `Resumed{by_engine}` and `FreezeFailed` events; it doesn't just issue commands.
- **Defaults** (user-configurable presets "Relaxed / Balanced / Aggressive"): Balanced = `T_freeze` 5 min, `T_discard` 30 min (D6).
- **Lazy session restore:** on startup, every restored tab except the active one starts **Discarded**.
- Background-opened tabs (middle-click) load normally, then become eligible for hibernation like any other tab.

### 7.2 Activity signals
Each signal is **trusted** (native engine API or our own UI) or **page-reported** (from injected JS, forgeable; §6.6). Backends document which signals they provide and from which source.

| Signal | Windows (WebView2) | macOS (WKWebView) | Linux (WebKitGTK) | Page-reported fallback (world) |
|---|---|---|---|---|
| Audible audio | ✅ trusted: `IsDocumentPlayingAudio` + event | `requestMediaPlaybackState` (verify) | ✅ trusted: `is-playing-audio` | media `play`/`pause` events (isolated); `AudioContext` (main) |
| User-muted | ✅ trusted: our `IsMuted` state | our state (mute is SPI or JS; D7) | ✅ trusted: our `is-muted` state | — |
| Capture (mic/camera) | start: `PermissionRequested` (trusted hint); end: page-reported | ✅ trusted: `microphoneCaptureState`, `cameraCaptureState` | `is-capturing-*` properties (verify) | `getUserMedia` wrapper + track `ended` (main) |
| Screen capture | `ScreenCaptureStarting` (verify) | `displayCaptureState` (verify) | (verify) | `getDisplayMedia` wrapper (main) |
| Fullscreen | ✅ trusted: `ContainsFullScreenElementChanged` | (verify) | (verify) | `fullscreenchange` (isolated) |
| Picture-in-picture | — | — | — | `enterpictureinpicture` (isolated) |
| Unsaved form input | — | — | — | `input` events per §7.3 rule 4 definition (isolated) |
| `beforeunload` registered | — | — | — | (main) |
| Open JS dialog / auth prompt / cert interstitial | ✅ trusted: our handlers | ✅ trusted | ✅ trusted | — |
| Notifications permission granted | ✅ trusted: our `site_settings` | ✅ | ✅ | — |
| Active download | ✅ trusted: download events | ✅ | ✅ | — |
| Last navigation was POST | ✅ trusted: navigation events (verify per engine) | (verify) | (verify) | — |
| Last user interaction in tab | ✅ trusted: our focus/input tracking | ✅ | ✅ | — |

### 7.3 Protection rules
A background tab is **protected** (never frozen or discarded) if any of these hold:

**Trusted rules:**
1. Audible audio now or in the last **5 minutes**, and not muted by the user. Muted/inaudible autoplay doesn't count.
2. Mic/camera/screen capture active, by a trusted source, or a page-reported capture that started after a trusted `PermissionRequested` grant.
3. An open JS dialog, file picker, auth prompt, certificate interstitial, or WebAuthn ceremony.
4. An active download.
5. The user marked it **Keep awake**, or its site is on the **Always keep awake** list (ships with a small editable default list, e.g. `music.youtube.com`, `discord.com`, `teams.microsoft.com`, `meet.google.com`).
6. It is the **previously active** tab, for the first 2 minutes after leaving it.
7. The site has **notification permission granted** (Gmail, Calendar, Slack): protected from **discard**, but freezing is allowed only if the user enables "freeze notification sites" (off by default, because freezing would delay reminders).

**Page-reported rules** count only if there was a **real user interaction in that tab** (trusted) after page load, and they **expire**:
8. **Unsaved input**: a field (not a search box, not a password-only field) whose value differs from its default, not submitted, still in the document, edited after user interaction. Expires after 24 h or after the tab has been idle 6 h (configurable).
9. Picture-in-picture, page-reported fullscreen, page-reported audio (where no native signal exists): expire 10 min after the last refresh message.

Every protected tab shows its reason on `lwb://memory` and in the tab tooltip, labelled "page says…" for page-reported reasons. The user can override any protection with "Hibernate now".

**Learning:** if the user re-activates a discarded tab within 60 s of it being discarded, twice for the same site, offer "Keep this site awake?" (non-modal).

Protection is re-evaluated on every signal change and at every policy wake-up.

### 7.4 Memory budget and policy scheduling
- User setting **Memory limit**: off / auto / custom. "Auto" = 30% of physical RAM, minimum 1.5 GB.
- **Event-driven scheduling:** the policy wakes at the next deadline (the next tab crossing `T_freeze`/`T_discard`, or a protection expiring), on any signal change, and on **OS memory-pressure events** (Windows `CreateMemoryResourceNotification`, macOS `DISPATCH_SOURCE_TYPE_MEMORYPRESSURE`, Linux PSI `/proc/pressure/memory`). Memory is sampled periodically (every 5 s) **only while a limit is set**.
- On each wake-up:
  1. Apply idle timers (Background → Frozen → Discarded) to unprotected tabs.
  2. If a limit is set and total memory > limit: **discard** the unprotected, non-active tab with the oldest last-active time (tie-break: largest attributed memory). Then wait a **10 s cooldown** before the next forced discard. Keep discarding until total < **90% of the limit** (hysteresis).
  3. If over the limit and every candidate is protected: **do nothing destructive**. Show a non-intrusive indicator ("Over memory limit: 3 protected tabs") that links to `lwb://memory`.
- All of this is a pure function in `lwb-core`: `fn plan(state: &BrowserState, now: Instant, mem: Option<MemSample>, cfg: &Policy) -> Plan { actions: Vec<TabAction>, next_wake: Option<Instant> }`. It is fully unit-tested (§10, Phase 2a).

### 7.5 Restore
- **Frozen → Active:** resume, instant.
- **Discarded → Active:** create a `Web` view, navigate to the saved URL, restore the scroll position via an injected script after load (best effort), and keep our stored per-tab history stack. Back/Forward on a restored tab navigates through our stack, since the engine's own history was lost.
- **Never auto-reload unsafe URLs:** if the tab's last navigation was a POST, or the site is marked "don't auto-reload" in `site_settings`, activating it shows a **"Click to reload"** placeholder instead.

### 7.6 Measuring memory (`lwb-memprobe`)
- **Metric per OS** (matches what users see in their task manager):
  - Windows: **private working set**, read cheaply via `GetProcessMemoryInfo` + `PROCESS_MEMORY_COUNTERS_EX2.PrivateWorkingSetSize` (no working-set page walks on the hot path).
  - macOS: **`phys_footprint`** via `proc_pid_rusage`.
  - Linux: **PSS** from `/proc/<pid>/smaps_rollup`.
- **Which processes are ours:** attribution is always by **process tree / engine API, never by process name**. Other apps (Teams, Outlook, widgets) run their own `msedgewebview2.exe` trees.
  - Windows: our process + `CoreWebView2Environment.GetProcessExtendedInfos` (also gives renderer→frame mapping for per-tab numbers; a shared renderer's memory is split and marked "shared").
  - macOS: **known limitation.** WebKit's WebContent/Networking/GPU processes are XPC services, not children, and there's no public API to map them to us or to views. Phase 0c picks one of: private `_webProcessIdentifier` / responsible-PID SPI (D7), or a heuristic (enumerate `com.apple.WebKit.*` processes for our user, matched by launch time). Per-tab numbers are shown only if the chosen method supports them, and labelled "estimate".
  - Linux: our process tree (WebKitGTK subprocesses are children).

## 8. Built-in features

### 8.1 Ad and tracker blocking (`lwb-blocker`)
- Engine: adblock-rust. Default lists: EasyList + EasyPrivacy (+ optionally uBlock Origin's base filters), **downloaded at runtime** (weekly, ETag, pinned HTTPS URLs) and cached compiled on disk. Scriptlets/redirects come only from the pinned bundle (§6.6).
- **Windows:** network-level blocking via WebView2 `WebResourceRequested`, registered with **request-source kinds that include service/shared workers** (`AddWebResourceRequestedFilterWithRequestSourceKinds`, (verify) the minimum runtime). Requests arrive on the UI thread, so the handler must be fast (a precompiled engine lookup, no allocation-heavy work), and page-load latency is benchmarked (§4 target). The adblock engine stays resident and counts toward the shell's 60 MB budget, so Phase 0b measures it.
- **macOS and Linux:** convert rules to WebKit **content rule list** JSON (adblock-rust's content-blocking conversion) → `WKContentRuleListStore` / `WebKitUserContentFilterStore`, split into several lists to stay under WebKit's per-list rule limit. Compile in a **short-lived worker**, then drop the engine. Precompute generic cosmetic CSS so no engine stays resident.
- Cosmetic filtering: inject generic + site-specific CSS.
- Per-site toggle in the address bar, blocked-count badge.
- Phase 3a also evaluates an alternative on Windows: a declarativeNetRequest (MV3) blocker loaded as a WebView2 browser extension, which runs inside Chromium with no per-request round trip. Licensing (GPL component loaded separately) needs a lawyer's check before shipping.
- No promises about YouTube ad blocking. It is an arms race, best effort only.

### 8.2 Dark mode
1. **Native preference:** tell pages the user prefers dark (`prefers-color-scheme: dark`). WebView2 `PreferredColorScheme`; WebKit follows the app/window appearance.
2. **Forced dark** for sites without a dark theme: injected CSS filter mode (invert + hue-rotate, re-inverting images and video), per-site toggle, off by default.
3. Browser UI: light, dark, or follow system.

### 8.3 Audio controls
- Per-tab **mute**: native on Windows (`IsMuted`) and Linux (`is-muted`). On macOS, public WKWebView has no per-view mute (verify); use private `_setPageMuted:` under D7, else a JS fallback that mutes media elements (misses Web Audio).
- Per-tab **volume boost** (0–400%) via injected Web Audio (`GainNode`). Cross-origin media without CORS can't be routed: detect it, fall back to 100%, and explain why in a tooltip.

### 8.4 Daily-driver features
**Phase 1 (needed to "browse normally for a day"):** address bar with search (configurable engine), back/forward/reload/stop, **keyboard shortcuts that work even when a page has focus** (native accelerator handling per backend: WebView2 `AcceleratorKeyPressed`, macOS menu key equivalents / `NSEvent` monitor, GTK key handlers). Shortcuts: Ctrl/Cmd + T, W, Shift+T, L, R, F, Tab/Shift+Tab, 1–9, +/−/0, Alt+←/→. Also: **our own page context menu** (incl. "Open link in new tab", Inspect when devtools are enabled), **permission prompts** with per-site storage, **element fullscreen** (chrome hides, view fills the window), **certificate error interstitial**, **HTTP auth dialog**, **JS dialogs** (alert/confirm/prompt, `beforeunload` on close), **HiDPI and moving between monitors** with different scale factors, **find in page**, **history recording** (data only; UI in Phase 4), per-site zoom, downloads list, crashed-tab page with Reload, DevTools (behind a setting).

**Phase 4:** history UI, bookmarks (+ bar, import from Chrome/Brave/Edge `Bookmarks` JSON), address-bar autocomplete, **private windows**, **clear browsing data**, **multiple windows** + moving tabs between them, **print** and **PDF viewing** (built into WebView2/WKWebView; WebKitGTK (verify), else fall back to opening externally), **default-browser registration** (Windows: installer registers capabilities, then opens the Default Apps page, since apps can't set themselves as default), **many-tab usability** (tab search, overflow with 50+ tabs, close duplicates), **Global Privacy Control** header and an **HTTPS-upgrade mode**.

**Ongoing baseline from Phase 1:** keyboard focus moves predictably between the chrome and content; chrome HTML is screen-reader labelled; IME input works in the address bar.

### 8.5 Updates
Phase 4b: on startup (at most daily), check the GitHub Releases API and **notify only**. Releases include an **ed25519 (minisign) signature** of each artifact, checked before telling the user an update is genuine. This works without paid OS code signing.

### 8.6 Crash diagnostics
WebView2 writes Crashpad dumps for its own processes. For our shell: a panic hook writing a log, plus a "Copy diagnostics" button on `lwb://about`. No automatic upload.

### 8.7 Extensions (Phase 5)
1. Built-in features first (§8.1–8.3).
2. **Our own lightweight extension format:** manifest + content scripts + CSS + a small, permissioned API (storage, tab info, messaging), mirroring a subset of WebExtensions so simple Chrome extensions port easily. Works on all three OSes.
3. **Windows only, experimental:** load Chrome extensions via WebView2's browser-extension API. Compatibility is partial and password managers are the hard case. Mark it clearly as Windows-only.

### 8.8 AI toolkit (Phase 6)
Optional and **off by default**. When off, it loads nothing and costs zero memory (verified by benchmark). Initial features: summarize page, ask about the page, "tidy my tabs", explain a selection. Providers: bring-your-own key (free), or prepaid **credits** via our proxy on the owner's server. Page content is sent only on explicit user action and never stored server-side beyond the request. Specified in `docs/AI-SPEC.md` before Phase 6.

## 9. Benchmark methodology (`tools/lwb-bench`)
- Launches each browser with a **fresh profile** and a fixed URL list, waits for loads to settle, then samples memory (§7.6 metrics, whole process tree) every 5 s for 5 min. Reports median and p90. 3 runs per configuration; report the median of the medians. Also records page-load times (for the blocker-overhead target).
- **Required browsers/configs:** ours (hibernation off; default preset), **Brave with Memory Saver Off, Moderate, Balanced (the default) and Maximum**, Chrome (same modes, if it still offers all of them), Edge (Windows). The bench sets the mode explicitly in each fresh profile, so the owner's personal browser settings don't affect results. Brave and Chrome also have an "Always keep these sites active" list; it stays empty in benchmark profiles, matching our empty-by-default user list except for the shipped defaults, which the results note. Headline comparisons use the default (Balanced); Maximum is always reported alongside, as the strongest competitor configuration. Optional extras: Firefox, Safari (macOS, manual). Fresh profiles, no extensions, default settings otherwise. Brave's built-in blocking stays on (noted in results).
- **Autoplay:** fresh profiles block unmuted autoplay, so the YouTube tab is started the same way in every browser: a scripted click. If impossible, use `--autoplay-policy=no-user-gesture-required` for all Chromium-based browsers and document it.
- **Tab sets** (`bench/tabs-standard.txt`, `bench/tabs-realistic.txt`, URLs fixed and versioned in Phase 0d):
  - *Standard (20, logged out):* YouTube video (playing), Wikipedia, two news sites, Reddit thread, GitHub repo, Stack Overflow, MDN, Google Maps, shopping product page, recipe site (ad-heavy), Twitch channel, a docs site, a blog, weather, sports, long-form article, image-heavy site, Excalidraw, a search results page.
  - *Realistic mix (20):* the standard set, but 5 tabs must stay alive: audio playing, a `getUserMedia` test page (simulated call), a page with a half-filled form, a chat-style app (live connection), a site with notification permission granted. Measured at 30 min. This metric shows what we do better than Memory Saver: our browser should use less memory **and** keep all 5 alive, and we report whether Brave/Chrome kept them alive.
- **Driving our browser:** a `bench`-feature-only control channel (CLI flags + a local socket bound to 127.0.0.1 with a random token) to open tab sets, fast-forward the policy clock ("30-min idle simulation") and read state. Never in release builds (§6.6).
- **Owner scenario:** a manual checklist reproducing the owner's usual logged-in Brave session, measured in both browsers.
- **Public claims** (from the Phase 2b write-up on): consider recording the tab set with a replay proxy (e.g. Web Page Replay) so runs months apart stay comparable.
- Output: CSV + markdown summary in `bench/results/<date>-<os>-<machine>.md`. Methodology must be documented and reproducible, since these numbers become marketing claims.

## 10. Phases

Each sub-phase = one branch (`phase-<id>-<slug>`) → PR to `main`, sized for **one implementation session** (two at most, with a handoff note). It is done when its acceptance criteria are met, CI is green, the manual checklist (`docs/testing/phase-<id>.md`, written by the implementer) has been run on Windows, and the review session plus the orchestrator approve it. macOS work is marked **"CI-built, not run"** in STATUS.md until the MacBook arrives.

### Phase 0: Foundation and spikes
**0a. Repo foundation** (small)
- Workspace skeleton (§6.2) with empty crates that compile; `rustfmt`, `clippy -D warnings`, `cargo-deny` (license allow-list + advisories), `.editorconfig`.
- `LICENSE` (FSL-1.1-ALv2, from the official text at fsl.software), CLA Assistant config, `CONTRIBUTING.md`, issue templates (bug, feature, tester feedback).
- CI: GitHub Actions matrix (windows-latest, macos-latest, ubuntu-latest): fmt, clippy, build, test, cargo-deny. Linux job installs the WebKitGTK dev packages.
- CLAUDE.md: add the real build/test commands.
- **Acceptance:** CI green on all three OSes; `cargo deny check` passes.

**0b. Windows engine spike** (the go/no-go for the primary platform)
In `spikes/engine-spike/` (throwaway quality is fine). One window: chrome view + 2 content views, switchable. Record a ✅/❌ table with notes and code pointers in `docs/spikes/phase0b-windows.md` for:
- multiple child webviews + bounds/visibility via wry; access to `ICoreWebView2*` from wry;
- one shared environment across views (confirm one browser process);
- `TrySuspend`/`Resume`, engine-initiated resume, `MemoryUsageTargetLevel`;
- `IsDocumentPlayingAudio` + event, `IsMuted`;
- `GetProcessExtendedInfos` (renderer→frame mapping);
- `WebResourceRequested` with worker request-source kinds; measured per-request handler latency and adblock-rust resident memory (EasyList + EasyPrivacy);
- `NewWindowRequested` with opener preserved: **real Google sign-in popup**;
- `AcceleratorKeyPressed` (Ctrl+T/L/W work while the page has focus);
- `ContextMenuRequested` (replace with our menu), `PermissionRequested`, `ContainsFullScreenElementChanged`, `ServerCertificateErrorDetected`, `BasicAuthenticationRequested`, `ScreenCaptureStarting`;
- whether host scripts can run in an isolated world;
- whether navigation events expose POST;
- custom protocol: actual URL form (`lwb.localhost`?) and blocking it in web views;
- **overlay test:** chrome view grown over the content area with a transparent background shows an autocomplete dropdown on top of a page, and shrinks back without flicker;
- HiDPI: bounds correct at 100%/150% and when dragging between monitors;
- memory: empty window total with and without the chrome view.
- **Exit:** every ❌ gets a DECISIONS entry (workaround / direct bindings / scope change) and a go/no-go for Windows.

**0c. macOS + Linux engine spike**
The same spike app and checklist, adapted, in `docs/spikes/phase0c-macos.md` and `phase0c-linux.md`. Extras: Linux on **Wayland and X11** via the GTK path; WebKit content rule lists; macOS media/capture state APIs; macOS mute and process attribution (pick a method under D7); WebKitGTK permission requests, PDF, `is-capturing-*`. Run on GitHub's macOS runners (which have a GUI session) for the automatable checks. Hands-on macOS checks wait for the MacBook. Can run in parallel with 0d.

**0d. Memory probe + benchmark v0 + baselines**
`lwb-memprobe` (Windows implementation first; macOS/Linux stubs following §7.6) and `lwb-bench` v0 (launch other browsers with fresh profiles + tab set, sample, report). Fix the two tab-set files. Produce Windows baselines for Brave (Memory Saver off/on), Chrome (off/on) and Edge on the owner's machine. The orchestrator then recalibrates §4.
- **Acceptance:** reproducible within ±10% across 3 runs; results committed in `bench/results/`.

### Phase 1: Core browser
**1a. Engine traits + Windows backend + browsing basics:** the `Engine`/`EngineView` traits (§6.3) and the Windows backend; view kinds and view swapping at the internal/web boundary (§6.6); window with chrome view (tab strip: new/close/switch/reorder/pinned; address bar: URL or search; back/forward/reload/stop; loading indicator; title + favicon); native accelerators; `lwb://newtab`, `lwb://about`; new-window requests → new tab (with opener popups); overlay mechanism (§6.5); HiDPI.

**1b. Storage + essentials:** `lwb-storage`; settings + `lwb://settings`; session save (debounced) + lazy restore (minimal Discarded state, §7.1); history recording; downloads; crashed-tab page; our page context menu; permission prompts + per-site storage; fullscreen; certificate interstitial; HTTP auth; JS dialogs; find in page; per-site zoom; DevTools setting.
- **Acceptance (end of 1b):** the owner browses normally on Windows for a day (YouTube, Gmail, GitHub, a banking site, Google sign-in popup) with no blocking bugs. Empty-window memory recorded.

**1c. macOS + Linux backends** to feature parity with 1a/1b where `EngineCaps` allow. Acceptance: builds and launches in CI; manual smoke test on Linux (Wayland); macOS "CI-built, not run" until the MacBook arrives.

### Phase 2: Memory management
**2a. Lifecycle, signals, protection:** full state machine with engine events (§7.1); trusted + page-reported signals (§7.2) per backend; all protection rules incl. expiry and the user-interaction requirement (§7.3); Keep-awake; Always-keep-awake list; restore with scroll/history and the POST "click to reload" rule (§7.5). `plan()` unit tests covering: idle timers, every protection rule, page-reported expiry, the podcast scenario, a forged "unsaved input" message without user interaction (must not protect), notification-site rule, previously-active grace, engine-initiated resume, freeze failure.

**2b. Budget + dashboard:** memory limit with presets, hysteresis, cooldown and OS pressure triggers (§7.4); event-driven scheduling; `lwb://memory` (total, per-process, per-tab where supported, states, protection reasons, "hibernate now"); tab-strip indicators; learning prompt. Tests: budget pressure discards (never freezes), the all-protected case, hysteresis, cooldown.
- **Acceptance (end of 2b):** realistic-mix target in §4 met or the gap explained; zero wrongly hibernated tabs in the scripted 60-min Aggressive-preset test; first public benchmark write-up.

### Phase 3: Built-in features
**3a. Ad/tracker blocking** (§8.1) on all three backends, list updates, cosmetic filtering, per-site toggle, badge, plus the MV3-extension evaluation on Windows. Acceptance: known-ad test page blocked on all 3 OSes; blocker page-load target (§4) met on Windows; benchmark rerun.

**3b. Dark mode + audio** (§8.2, §8.3). Acceptance: forced dark is usable on 10 sample sites; volume boost works on YouTube; mute works on all 3 OSes (macOS via D7 decision).

### Phase 4: Daily-driver polish and tester release
**4a. Features:** everything in the §8.4 Phase 4 list, final name/icon if decided (D2), About page with donation/sponsor links.

**4b. Release engineering:** packaging with `cargo-packager` (Windows NSIS/MSI unsigned; macOS `.dmg` with an ad-hoc signed `.app`; Linux `.deb` + AppImage); checksums + minisign signatures; update notifier (§8.5); crash diagnostics (§8.6); `docs/TESTERS.md` (§11); in-app "Send feedback" link.
- **Acceptance:** builds install on a clean Windows 11 machine (incl. the Smart App Control note) and on macOS by following `TESTERS.md`; first 5–10 external testers onboarded.

### Phase 5: Extensions (§8.7)
### Phase 6: AI toolkit and credits (§8.8, after `docs/AI-SPEC.md`)
### Phase 7: Growth and revenue
Website, search-partnership outreach, sponsorships, code signing + notarization once tester feedback is positive, auto-update, optional sync.
### Phase 8: Engine research (decision gate)
Evaluate Servo as an alternative backend behind the `Engine` trait, and decide whether owning an engine is worth it. Output: a report and recommendation, not a commitment.

## 11. Distribution before code signing
- Windows builds are unsigned. macOS builds are ad-hoc signed (Apple Silicon requires at least an ad-hoc signature; `codesign --force --deep -s -` on the `.app`) and not notarized.
- `TESTERS.md` explains: Windows SmartScreen ("More info" → "Run anyway"); **Smart App Control** (on clean Windows 11 installs it can block unsigned apps outright with no "Run anyway"; testers with SAC on can't run our builds until we sign); macOS ("Open Anyway" in System Settings → Privacy & Security, or `xattr -dr com.apple.quarantine`); how to verify checksums and signatures; what data we collect (none).
- **Signing decision (D8):** paid signing waits for positive tester feedback, *unless* Smart App Control blocks a meaningful share of testers. Then evaluate Microsoft's low-cost cloud signing service (roughly $10/month for eligible individuals at last check; verify current name, price and eligibility at that time).

## 12. Risks

| Risk | Impact | Mitigation |
|---|---|---|
| wry doesn't expose multi-view or platform hooks we need | High | Spikes 0b/0c; direct `webview2-com`/`objc2`/`webkit2gtk` bindings behind our traits |
| OAuth popups break (no `window.opener`) | High (logins fail) | Explicit spike test with a real Google sign-in |
| Overlays (dropdowns) can't draw over content | High (UX) | Chrome-view growth tested in 0b; overlay-view fallback |
| On Windows we run the same Chromium as Brave/Chrome | Medium | Honest targets (§4); win on lazy restore, smart protection, transparency; bigger wins on macOS |
| Brave/Chrome already have Memory Saver | Medium | Benchmark against it; compete on *not* breaking in-use tabs (realistic-mix metric) + transparency |
| Pages forge "protected" signals | Medium | Trusted vs page-reported split; interaction requirement; expiry (§7.3) |
| macOS process attribution needs private API | Medium | D7 policy; totals-only fallback |
| Linux Wayland child views | Medium | GTK path; tested in 0c |
| Ad-block handler slows page loads on Windows | Medium | Fast handler, latency benchmark, MV3-extension alternative (3a) |
| Hijacked filter list runs scripts | High | Pinned list URLs; scriptlets only from a bundled resource set |
| Hibernating something the user needed | High (trust) | Conservative rules, grace periods, Keep-awake, learning prompt, zero-tolerance metric |
| Unsigned builds / Smart App Control block testers | Medium | `TESTERS.md`, signatures; D8 trigger for cheap signing |
| Ad-blocking arms race (YouTube) | Low/Medium | Best effort, no promises |
| License misunderstanding | Medium | FSL + CLA, lawyer review before commercialisation |

## 13. Open decisions and questions (owner)
| # | Decision | Recommendation | Status |
|---|---|---|---|
| D1 | License | FSL-1.1-ALv2 + CLA | **Decided** (DEC-001) |
| D2 | Product name | TBD; check trademark/domain availability | Open (needed by Phase 4a) |
| D3 | Repo visibility | Public | **Decided** (DEC-002) |
| D4 | Default search engine | Configurable; pick the default when a partnership exists | Open (Phase 4) |
| D5 | Overlay mechanism | Grow the chrome view (§6.5) unless 0b says otherwise | Pending 0b |
| D6 | Default hibernation preset | Balanced (5 min freeze / 30 min discard) | Open |
| D7 | Private Apple APIs (mute, process attribution) | Allowed behind the `macos-spi` feature + runtime check + public fallback | **Decided** (DEC-004) |
| D8 | Earlier cheap Windows signing | Only if Smart App Control blocks testers (§11) | Open (Phase 4b) |
| Q1 | Was Brave's Memory Saver on in the owner's baseline? | — | **Answered:** off; 8.1 GB / 64 processes |

## 14. Workflow (orchestration)
1. **Spec review:** fresh session(s) review this spec; findings go in `docs/reviews/`. The orchestrator folds them in, and the owner approves → status `APPROVED v1.0`.
2. **Per sub-phase:** the owner starts a local implementation session with `docs/prompts/phase-<id>.md` (written by the orchestrator, normally once the previous sub-phase is merged). The implementer works on `phase-<id>-<slug>`, keeps `docs/STATUS.md` updated, records deviations in `docs/DECISIONS.md`, and opens a PR.
3. **Review:** a fresh review session checks the PR against this spec's acceptance criteria and §6.6 security rules (prompt: `docs/prompts/review-pr.md`). The orchestrator does a final pass; the owner merges.
4. If a session's context gets long, it stops at a clean point and writes a handoff note in `docs/STATUS.md`.
