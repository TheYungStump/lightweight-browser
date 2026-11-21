# Lightweight Browser — Project Specification

> **Status:** DRAFT v0.3 — v0.2 (review 1 incorporated) plus the owner's positioning decisions from the research report `docs/research/2026-11-browser-positioning.md` (DEC-008 to DEC-013).
> **Working name:** "Lightweight Browser" (crate prefix `lwb`). The final name is an open decision (§13). Renaming is cheap until Phase 4.
> **Owner:** project owner (human). **Orchestrator:** cloud Claude session. **Implementers:** local Claude Code sessions, one per sub-phase.

This document is the single source of truth. Implementation sessions must read it before writing code, and must not silently diverge from it. If reality forces a change (an API doesn't exist, a design doesn't work), record it in `docs/DECISIONS.md` and flag it in the phase PR.

Items marked **(verify)** are believed correct but must be confirmed in the Phase 0 spikes before anything is built on them.

---

## 1. Vision

**Headline: "The most memory-efficient browser that never loses your work."** (DEC-008)

Saving memory alone doesn't set us apart. Brave, Chrome and Edge already discard background tabs ("Memory Saver", "Sleeping tabs"). The owner's own test (Windows Task Manager, usual tabs): Brave at **8.1 GB** (64 processes) with Memory Saver off dropped to **2.6 GB** on Maximum. Chrome's Memory Saver (and Brave, which inherits it) also already exempts some in-use tabs: audio/video playback and calls, screen sharing, page notifications, active downloads, partially filled forms, pinned tabs, connected devices ([Chrome help](https://support.google.com/chrome/answer/12929150)). But when a tab is discarded anyway, every shipping discarder brings it back as a reload from the URL, and typed text and position are typically lost (whether Chromium's reload restores some fields is measured in 0d). Users report turning the feature off after losing work (research report; this rests on search extracts and is **unverified** until the Phase 7 spot-check).

Our difference: **like Memory Saver, we don't sleep tabs you're using, but we show you why each tab is awake, honour the memory limit you set without sleeping protected tabs, and when a tab is slept anyway it comes back with your typed text and position** (best effort, §7.5). Hibernation safe enough to leave on.

**Headline status:** the headline is the owner's decision (DEC-008). It is confirmed for public use only after 0d has run the realistic mix and the forced-discard check against Brave and Edge (§9), and public wording is scoped to what the benchmark measures (e.g. "as lean as Brave on Maximum, and it never loses your work"), not an unqualified superlative (DEC-014).

How we get there:
1. **Don't sleep tabs holding work** (the primary guarantee, §7.3): a podcast playing in the background, a video call, a half-filled form, a download, the tab you just left. Every protected tab shows why it is protected.
2. **Restore what we can when a tab was slept anyway** (§7.5): supported form fields and scroll position (best effort), and a screenshot while the page reloads. We're honest about the limits.
3. **Lean by design:** the operating system's own web engine instead of another bundled Chromium (a real gain on macOS and Linux; on Windows, WebView2 is itself Chromium, so the gains there come from tab management), lazy session restore, and freezing/unloading idle background tabs.
4. **Transparency as the proof:** `lwb://memory` shows where memory goes and why each tab is awake or asleep, and our public benchmark compares against competitors *with their savers at the most aggressive setting* (§9).
5. Built in, so nobody needs extensions for them: ad/tracker blocking, dark mode, volume boost.

**Brand tone: calm.** No AI unless the user turns it on (§8.8), no crypto, no shopping features, no news feeds, no telemetry by default. The "no telemetry" claim covers our code. On Windows, Microsoft's WebView2 runtime sends **required diagnostic data that no app can turn off**; optional data follows the Windows "Diagnostics & feedback" setting, and the embedder controls only crash reporting and SmartScreen ([WebView2 data and privacy](https://learn.microsoft.com/microsoft-edge/webview2/concepts/data-privacy)). So the public wording is: "We collect nothing. On Windows, Microsoft's WebView2 runtime sends required diagnostic data that no app can turn off." Phase 0b confirms the details; we keep crash dumps local (§8.6) and SmartScreen is D16.

**First audiences:**
- people with many tabs open (on 8–16 GB machines) who are frustrated by browser memory use, or who turned Memory Saver off because it lost their work;
- people on older or low-RAM PCs: 8 GB machines, and Windows 10, which keeps WebView2 runtime updates until at least October 2028 (research report). This is an audience, not the differentiator: they can already turn on Brave Maximum.

**Supporting claims only, never headlines:** battery life, privacy, built-in ad blocking.

## 2. Goals and non-goals

### Goals
- G1. Daily-driver-quality browsing on Windows, macOS and Linux desktop.
- G2. Memory use **at or below Brave and Edge with their memory savers on the most aggressive setting**, on a reproducible benchmark (§9), with published results.
- G3. **Never lose the user's work:** tabs that are in use are never hibernated (§7.3), and a tab that was hibernated anyway comes back with its supported form fields and scroll position, best effort and within published limits (§7.5). Smart hibernation and a user-set memory budget make this the default, not a setting to fear.
- G4. Built-in ad/tracker blocking, dark mode, per-tab mute and volume boost.
- G5. Calm and private by default: no telemetry unless the user opts in; no AI unless the user turns it on; no crypto, shopping features or news feeds; private windows; Global Privacy Control.
- G6. An architecture that lets us swap or add web engines later (e.g. Servo) without rewriting the browser.
- G7. Heavy-tab usability: a 500-tab session restores reliably, tab search covers sleeping tabs, optional auto-archive, spaces.
- G8. Later: an extension system, then optional AI behind one slider that **defaults to "No AI"** and costs **zero memory** there (§8.8). Updates never turn AI on.

### Non-goals (for now)
- Writing our own HTML/CSS/JS engine. Phase 8 is the decision gate for that.
- Mobile (iOS/Android). Keeping `lwb-core` pure Rust keeps it reusable if a mobile app happens later. iOS requires WebKit (outside the EU/Japan exemptions), which our engine abstraction already supports.
- Full Chrome Web Store compatibility on all platforms (see §8.7).
- Built-in password manager or sync (users keep their existing password manager; sync is a later paid-tier candidate).
- Paid code signing before positive tester feedback (§11).
- AI agents that act on pages (click, type, submit, navigate on the user's behalf), until D9 decides otherwise.
- A bundled local AI model by default (it would contradict the memory headline; D13).

## 3. Platforms

| Platform | Minimum | Engine | Priority |
|---|---|---|---|
| Windows | Windows 10 22H2 (WebView2 runtime updates until at least Oct 2028; §12), Windows 11 (x64; ARM64 later) | Microsoft Edge WebView2 (Evergreen runtime) | **Primary**: owner's main machine |
| macOS | macOS 13 Ventura+ (Apple Silicon; Intel best-effort). Separate-login spaces need macOS 14+ (D12) | WKWebView | High (no hands-on testing until the owner's MacBook arrives) |
| Linux | Ubuntu 22.04 / 24.04 desktop, **Wayland and X11** | WebKitGTK (webkit2gtk-4.1), via wry's GTK path | Medium |

The owner's Ubuntu machine is a server. Linux support targets *desktop* Linux. The server can host backend services later (update manifest, feedback intake, AI credit proxy).

## 4. Success metrics

> All targets are **provisional until Phase 0d** produces real baselines on the owner's machine. Then the orchestrator recalibrates them and records the result in `DECISIONS.md`.

Measured with the benchmark in §9, same machine, fresh profiles. **The headline comparison is against Brave and Edge with their memory savers at their most aggressive setting** (Brave: Memory Saver Maximum; Edge: sleeping tabs at its most aggressive configuration, fixed in 0d), not their defaults (DEC-008).

| Metric | Target |
|---|---|
| Empty window (one new-tab page), total memory | ≤ 200 MB Windows, ≤ 150 MB macOS (tight on Windows: chrome-view and new-tab renderers plus WebView2's browser, GPU, network and utility processes; 0d reports the per-process breakdown) |
| Our shell process (Rust code incl. resident ad-block engine) | ≤ 60 MB |
| Standard 20-tab set, hibernation **off**, **Windows** | ≤ 100% of Brave, ≤ 85% of Chrome |
| Standard 20-tab set, hibernation **off**, **macOS** | ≤ 85% of Brave |
| **Realistic mix** (§9: 20 tabs, 5 of which must stay alive), 30 min, default preset — **headline** | ≤ 100% of Brave on **Maximum** and ≤ 100% of Edge at its **most aggressive** setting, **with all 5 must-stay-alive tabs alive** and the half-filled form's contents intact after a forced discard. Results report how many of the 5 tabs Brave and Edge kept alive |
| Realistic mix, second line | ≤ 35% of Brave with Memory Saver off (owner's choice; note it is looser than the ~32% Brave Maximum already reached on the owner's session, so 0d recalibrates it) |
| **Form restore** (§10 Phase 2b form-restore test suite; "comes back" as defined there) | **100%** of the supported fields in the suite come back after a forced discard. A real-site corpus score is reported separately, with no target. "100%" is never used in public copy |
| Owner's real-world usage (Brave 8.1 GB with Memory Saver off, 2.6 GB on Maximum) | ≤ Brave Maximum in the same logged-in session, reporting how many tabs each kept alive; recalibrated from 0d |
| Restoring a discarded tab (time to first paint, typical news site, cached) | ≤ 1.5 s |
| Page-load time cost of our ad blocker (Windows, median over the standard set) | ≤ +5% vs blocker off |
| Wrongly hibernated tabs (a tab hibernated while protected per §7.3 by a **trusted** signal) | **0**. Treat any occurrence as a bug |
| Installer / download size | ≤ 15 MB Windows and macOS (the engine is part of the OS), ≤ 10 MB Linux `.deb`, **including the AI module if it is bundled** (D11). For comparison, Chromium-based browser installers are typically 100 MB+ |
| Cold start to a usable window (warm OS cache, empty session) | ≤ 1 s |
| Keystroke to screen in the address bar | ≤ 16 ms |

**Every target states its statistic** (median, p95, or max) and its measurement method in `bench/README.md`, written in 0d. **Budgets that can be measured reliably on CI runners** (installer size, shell binary size, cold start, shell-process memory with an empty window) **are checked in CI from the phase that makes them measurable, and a regression fails the build.** Shared runners are noisy, so CI checks use medians over many runs with a stated tolerance. Whole-browser memory targets are measured on the owner's machine with `lwb-bench`, not in CI.

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
3. **AI credits** (Phase 6). Prepaid credits for the optional AI features, served through our own proxy, for users who move the AI slider above "No AI" (§8.8). "Bring your own API key" stays free; on-device OS models may also be free (D13). AI is never turned on to grow this revenue line: the default stays "No AI".
4. Possibly later: a paid tier (sync, premium themes, priority support).

Hard rules: no ads injected into pages, no selling user data, no telemetry without opt-in, **no update ever turns AI on** (§8.8). A public "How we're funded / what we collect" page states these (Phase 4d).

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
    fn clear_browsing_data(&self, what: DataKinds, profile: ProfileId) -> Result<()>;
}

pub enum ViewKind { Chrome, Internal, Web }           // see §6.6 privilege rules
pub struct ProfileId { pub space: SpaceId, pub private: bool } // opaque; SpaceId::DEFAULT until 5a.
                                                      // private = ephemeral data store. In the trait from 1a,
                                                      // so spaces (5a) don't refactor every call site
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

`snapshot()` (an image of the visible page, for the restore screenshot in §7.5) is added in Phase 2b (WebView2 `CapturePreview`, WKWebView `takeSnapshot(with:)`, WebKitGTK `webkit_web_view_get_snapshot`; all reachable from Rust). Assume it works **only on a visible view**: WebView2 doesn't render a view with `IsVisible = false`, and 2b measures the other two. Per-space data stores (§8.10) and Chrome-extension capabilities are added when first needed (Phase 5a / 5b), not before.

### 6.4 Process model
- **Windows:** create **exactly one `CoreWebView2Environment`** and pass it to every view. Private windows and spaces are **controller options within it** (`IsInPrivateModeEnabled`, `ProfileName` in `ICoreWebView2ControllerOptions`), so they share one browser process instead of each needing its own user-data folder and process tree. Views share the browser process only if the user-data folder *and* environment options match exactly. One mismatched `AdditionalBrowserArguments` means a second browser process or a creation error. Renderers are per-site (Chromium site isolation). Experiment flag (bench-only, off by default): `--renderer-process-limit=N`. Separate-login spaces (Phase 5a) add **WebView2 profiles** inside the same environment, which share its browser process; wry already exposes `with_profile_name`. Measure each extra profile's memory cost then. Environment-wide options (e.g. `IsCustomCrashReportingEnabled`, §8.6) are set once, on this environment.
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
- **Second layer:** every IPC message is checked against the sender's origin and frame where the engine reports them (WebView2 message args; macOS `WKScriptMessage.frameInfo`). Messages from the wrong origin or from subframes are dropped and logged. **WebKitGTK's script-message signals don't identify the sending frame or origin** (confirm in 1d), so there the handlers for internal pages and the page channel are registered **only in our isolated world** (`register_script_message_handler_in_world`), out of reach of page scripts, and `EngineCaps` reports the second layer as unavailable.

**Injected scripts and page-reported signals:**
- Features that need scripts in web pages (form/media detection fallbacks, form capture and restore (§7.5), scroll restore, volume boost, cosmetic filtering, forced dark mode) use a narrow **page channel**, separate from privileged IPC.
- Scripts run in an **isolated world** where available (`WKContentWorld`, WebKitGTK script worlds). Some fallbacks need the **main world**, where the page can tamper with them: wrapping `getUserMedia`, observing `AudioContext`, detecting `beforeunload` handlers. **On WebView2, assume host scripts run in the main world** (`ExecuteScript` and `AddScriptToExecuteOnDocumentCreated` run in the document, with no documented isolated-world option). Phase 0b confirms this and evaluates an alternative: shipping our page scripts as content scripts of a bundled unpacked extension (`AreBrowserExtensionsEnabled` + `AddBrowserExtension`), which Chromium runs in an isolated world; open questions are how it reaches the host and loading it into every profile and space. Assume any page script can call `chrome.webview.postMessage`, and can see, patch, forge or stall our main-world scripts.
- Therefore **every page-channel message is untrusted and forgeable.** It is schema-validated, rate-limited, scoped to its own tab, and can only produce *page-reported* signals (§7.2), which have weaker protection power than trusted ones (§7.3).

**Filter lists:** list URLs are pinned in code and HTTPS-only. List contents are untrusted input. **Scriptlet / redirect resources come only from a pinned resource bundle shipped with the app**, never from downloaded lists, so a hijacked list can't run JS on every site.

**Other rules:**
- Allowed address-bar schemes: `http`, `https`, `lwb` (internal), `file` (with a warning), `about:blank`. Others prompt or are refused.
- Permission requests (camera, mic, location, notifications, clipboard, …) always go through **our** prompt, never the engine default (WebKitGTK denies by default when unhandled; WebView2 shows its own UI). Decisions are stored per site in `site_settings`.
- Certificate errors → our interstitial with "go back" as the default. Never auto-proceed.
- Downloads go to a user-chosen folder and are never auto-opened. Make sure Mark-of-the-Web (Windows) / `com.apple.quarantine` (macOS) is set on downloaded files, setting it ourselves if the engine doesn't.
- `NewWindowRequested`: open as a new tab, **except** popups that need `window.opener` (OAuth "Sign in with Google/Microsoft/Apple"). These keep the opener relationship (WebView2: assign `NewWindowRequestedEventArgs.NewWindow`; WebKit: return the new view from `createWebViewWith` / `create` signal). The popup view **inherits the opener's setup**: `ViewKind::Web` (no IPC), blocker filters, navigation policy, permission handler, page scripts, and the **same profile/space and private flag** (wry: `NewWindowResponse::Create { webview }`, built on the opener's environment / data store). Phase 0b/0c must verify this works with a real Google sign-in, including a popup from a private window; 5a repeats it from a non-default space.
- **Fuzzing:** every parser of untrusted input has a `cargo-fuzz` target from the phase that introduces it: page-channel messages (incl. form-capture payloads), IPC messages, filter-list loading, bookmark/history/session importers, the single-instance URL handoff (§8.4), `site_settings` values from disk. Fuzz targets run briefly in CI and longer locally before a release.
- The benchmark control channel (§9) is compiled only with a `bench` Cargo feature that release builds never enable, and is never reachable from web content.
- Private Apple APIs (SPI) are allowed only with a DECISIONS entry, behind a runtime `respondsToSelector:` check with a public fallback, and only inside code gated by the **`macos-spi` Cargo feature** (on for direct-download builds, off for any future Mac App Store build) (D7, DEC-004).

### 6.7 Command layer
Every user-visible action is a **named command** dispatched through one place in `lwb-core`: `tab.new`, `tab.close`, `tab.activate`, `tab.keep_awake`, `tab.hibernate`, `nav.go`, `nav.back`, `find.start`, `zoom.set`, `settings.set`, `blocker.toggle_site`, and so on. The chrome view, keyboard accelerators, native menus and (later) the command palette all emit commands; there are no hidden actions.

Why it's worth doing from Phase 1a:
- **One path for every action**, so shortcuts, menus and the UI can't drift apart, and user-editable shortcuts are simply a keymap → command table.
- **Testability:** command sequences can be replayed against `lwb-core` in unit tests, and the `bench` control channel (§9) is just a command transport instead of a separate API.
- **Future surfaces at no extra cost:** a command palette (Phase 4b), and AI features (Phase 6) that change *browser* state (e.g. applying a tab-tidy suggestion the user accepted) going through the same commands, so everything they do is visible, undoable where possible, and permission-checked. Acting *on pages* (clicking, typing, submitting, navigating on the user's behalf) is not a command any AI feature may issue (D9).

Commands are a versioned, serde-serialisable enum with typed parameters. Each command declares which callers may invoke it: chrome view, internal page, keyboard, bench transport, or (later) external automation. **No external automation surface (CLI control, JSON over a socket, MCP) ships enabled, and no in-browser agent acts on pages.** Adding either is a separate decision (D9), because automating a logged-in browser is a serious security risk (prompt injection is unsolved; research report). The `bench`-feature control channel (§9) is a dev-only tool that release builds never contain, so it is outside D9.

### 6.8 Storage
SQLite file in the per-OS app data dir (`directories` crate), WAL mode. Tables (v1): `sessions`, `tabs` (incl. per-tab back/forward stack, scroll position, last-navigation-was-POST flag, `archived` flag + time for auto-archive), `history`, `bookmarks`, `site_settings` (zoom, dark mode, keep-awake, blocker exceptions, permissions), `settings`. Session state is **saved on every change (debounced ~1 s)**, incrementally (only changed rows), not only on exit, so crashes lose nothing and a 500-tab session stays cheap to save and restore. **Captured form data (§7.5) is never written to SQLite** (in memory only; D10). Engine profile data (cookies, cache) stays in the engine's own data store, pointed at our app data dir. **Private windows** use an ephemeral engine data store, and none of their tabs or history is written to SQLite. Migrations are versioned (`PRAGMA user_version`).

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
| Audible audio | ✅ trusted: `IsDocumentPlayingAudio` + event. It stays true while `IsMuted` (verify), so "playing" ≠ "audible" | `requestMediaPlaybackState` (macOS 12+) is **polled** and reports *playing*, not *audible* (muted autoplay counts). Audible needs SPI `_isPlayingAudio` KVO (D7) (verify) | ✅ trusted: `is-playing-audio` | media `play`/`pause` events (isolated); `AudioContext` (main) |
| User-muted | ✅ trusted: our `IsMuted` state | our state (mute is SPI or JS; D7) | ✅ trusted: our `is-muted` state | — |
| Capture (mic/camera) | start: `PermissionRequested` (trusted hint); end: page-reported | ✅ trusted: `microphoneCaptureState`, `cameraCaptureState` (KVO, macOS 12+) | ✅ trusted: `camera-capture-state`, `microphone-capture-state` (2.34+) | `getUserMedia` wrapper + track `ended` (main) |
| Screen capture | start: `ScreenCaptureStarting` (trusted, on `CoreWebView2` and `CoreWebView2Frame`); end: page-reported | `_displayCaptureState` is SPI (D7); public path is page-reported | ✅ trusted: `display-capture-state` (2.34+) | `getDisplayMedia` wrapper (main) |
| Fullscreen | ✅ trusted: `ContainsFullScreenElementChanged` | ✅ trusted: `fullscreenState` KVO (verify it's available on macOS 13) | ✅ trusted: `enter-fullscreen` / `leave-fullscreen` signals (verify) | `fullscreenchange` (isolated) |
| Picture-in-picture | — | — | — | `enterpictureinpicture` (isolated) |
| Unsaved form input | — | — | — | `input` events per §7.3 rule 4 definition (isolated) |
| `beforeunload` registered | — | — | — | (main) |
| Open JS dialog / auth prompt / cert interstitial | ✅ trusted: our handlers | ✅ trusted | ✅ trusted | — |
| Notifications permission granted | ✅ trusted: our `site_settings` | ✅ | ✅ | — |
| Active download | ✅ trusted: download events | ✅ | ✅ | — |
| Last navigation was POST | ✅ trusted: the method of document requests in our `WebResourceRequested` handler; `NavigationStarting` doesn't expose it (verify in 0b) | ✅ trusted: `WKNavigationAction.request.httpMethod` | ✅ trusted: the navigation action's request (`webkit_uri_request_get_http_method`) | — |
| Last user interaction in tab | ✅ trusted: our focus/input tracking | ✅ | ✅ | — |

### 7.3 Protection rules
**"Never loses your work" has two layers, in this order of importance (DEC-009):** (1) don't sleep tabs holding work: these protection rules, which are the primary guarantee and what the public claim rests on; (2) restore what we can when a tab is slept anyway (§7.5).

A background tab is **protected** (never frozen or discarded) if any of these hold:

**Trusted rules:**
1. Audible audio now or in the last **5 minutes**, and not muted by the user. Muted/inaudible autoplay doesn't count.
2. Mic/camera/screen capture active, by a trusted source, or a page-reported capture that started after a trusted `PermissionRequested` grant.
3. An open JS dialog, file picker, auth prompt, certificate interstitial, or WebAuthn ceremony.
4. An active download.
5. The user marked it **Keep awake**, or its site is on the **Always keep awake** list (ships with a small editable default list, e.g. `music.youtube.com`, `discord.com`, `teams.microsoft.com`, `meet.google.com`).
6. It is the **previously active** tab, for the first 2 minutes after leaving it.
7. The site has **notification permission granted** (Gmail, Calendar, Slack): protected from **discard**, but freezing is allowed only if the user enables "freeze notification sites" (off by default, because freezing would delay reminders). On macOS, WKWebView has no Web Notifications API, so "permission granted" means our injected `Notification` shim's permission (§8.4); the rule is the same, but the signal is our own state for a page-reported API.

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
- **Discarded → Active:** create a `Web` view, navigate to the saved URL, restore captured page state (below), and keep our stored per-tab history stack. Back/Forward on a restored tab navigates through our stack, since the engine's own history was lost.
- **Never auto-reload unsafe URLs:** if the tab's last navigation was a POST, or the site is marked "don't auto-reload" in `site_settings`, activating it shows a **"Click to reload"** placeholder instead.

**Capturing page state (layer 2 of "never loses your work", DEC-009, DEC-014).** A tab can still be discarded: budget pressure (§7.4), the user's "Hibernate now", an expired page-reported protection, or the idle timer on an unprotected tab. So we capture what a reload would lose:
- **What:** only fields the **user edited**: after a trusted user interaction, with a value that differs from its `defaultValue` / `defaultChecked` / `defaultSelected` (the same definition as §7.3 rule 8). Plus the focused field and the scroll position, and the page's normalized URL and a fingerprint of each form's field set (for the restore checks below). Fields are recorded with stable selectors (id, name, label, form and field position) and, for same-origin frames, the engine frame id. Covered: text-like inputs, `textarea`, `select`, checkboxes and radio buttons, and `contenteditable` (as plain text), in the top frame and same-origin frames.
- **When:** form state and scroll position are captured when a tab is frozen (the page can't change while frozen), or just before discarding a tab that wasn't frozen. If the engine resumed a frozen tab by itself (`Resumed{by_engine}`), the freeze-time capture is stale, so capture again before discarding. 0b checks whether `ExecuteScript` resumes a suspended WebView2 view.
- **Never blocks a discard:** capture is an async round trip through a channel the page can tamper with (main world on WebView2, §6.6). It has a fixed timeout (e.g. 500 ms); on timeout we discard anyway and log it. Under budget pressure, candidates with no edited fields are preferred.
- **Never captured:** password fields, including any field that was **ever** `type="password"` (a "show password" toggle) or has `autocomplete="current-password"`; fields whose `autocomplete` is `cc-*`, `one-time-code` or `new-password`; hidden fields; file inputs; cross-origin frames (this also leaves out embedded payment forms). Fields with `autocomplete="off"` and `inputmode="numeric"` (common for CVCs and PINs) are skipped by default, with a setting. **Nothing at all in private windows** (no form data, no screenshot).
- **Untrusted input:** captured data arrives over the page channel (§6.6), so it is schema-validated (fuzz target), size-capped per field, per tab and **globally across all tabs** (oldest-first eviction; caps set in 2b), and keyed by (profile/space, tab). It is never restored across spaces or between private and normal windows; moving a tab across those boundaries drops its captures.
- **Storage:** form data stays **in memory only**, in the shell process, and is dropped when the tab closes. It does not survive a browser restart (whether it should, encrypted with DPAPI / Keychain / libsecret, is D10). The promise is "not written to disk by us": process memory can still reach the OS pagefile or hibernation file. *Clear browsing data* (all, or per site) also clears captures and screenshots.

**Screenshots** (shown while a reloaded page loads):
- **Taken while the tab is active and visible**, debounced (e.g. 2 s after scrolling or input goes quiet, at most every 30 s), because a hidden view may not render (§6.3). Switching away uses the latest one, so tab switches stay instant.
- **Not taken** when the page has a non-empty field from the never-captured list, when focus or input went into a cross-origin frame since load, on sites in an editable "sensitive sites" list (banking, health; short or empty by default), or in private windows. A plain placeholder is shown instead.
- **Stored on disk, not in RAM:** downscaled and compressed (about viewport size or smaller, JPEG/WebP), **encrypted with a random per-run key held only in memory** (AES-GCM or ChaCha20-Poly1305), in the OS **local cache folder** (`ProjectDirs::cache_dir`: Local AppData on Windows, never Roaming; `~/Library/Caches`; `$XDG_CACHE_HOME`) with user-only permissions, under a size limit with LRU eviction. Deleted when the tab closes; after a crash the files are unreadable and the startup purge only tidies up.
- Shown **in the chrome view** (the grown-chrome mechanism, §6.5), not a new `Internal` view, so restoring doesn't cost a renderer. It is dimmed or marked "Restoring…", hidden from screen readers (which hear "restoring"), and blocks clicks and typing so input isn't lost into a picture. The chrome view gets it through a tab-scoped handle (`lwb://snapshot/<opaque-id>`), never a file path; web views can't reach it (they block `lwb.localhost`).

**Restoring safely** (each rule is tested in the 2b suite):
1. Restore only if the reloaded page's **normalized URL matches** the captured one (origin alone isn't enough) **and** the form's field-set fingerprint matches. Otherwise don't restore; show a non-modal **"We saved text you typed here: view / copy"** bar.
2. Write a field only if its current value still equals its post-load default. If the page loaded different content (e.g. a newer server draft), it's a **conflict**: skip it and offer it through the bar. Never overwrite.
3. Fire `input` only for text-like fields. **Never fire `change` automatically** on selects, checkboxes or radios; restore those only when no `change`-driven navigation, submit or fetch is detectable, and otherwise offer them through the bar. Never restore into a form whose `change` triggers navigation.
4. `contenteditable` (rich-text editors keep their own model): never written into the DOM. The text is offered through the bar (copy / insert); "insert" uses `document.execCommand('insertText')` after the user focuses the field, which editors treat as user input. No HTML is restored, so no sanitizer is needed.
5. The restore script checks the frame's `location.origin` **at the moment it writes**, and addresses same-origin frames by engine frame id, so a navigation between our check and the write can't redirect the data.
6. Wait for fields rendered after `load` (SPAs), but **stop as soon as the user types or clicks in the tab**, and give up after a fixed timeout (e.g. 10 s).
7. After the fields, restore the focused field and the scroll position.

**Honest limits** (stated in the UI help and on the public "never lose your work" page). A reload can't bring back:
- state that lived only in the page's memory, such as unsaved work held by a web app's own JavaScript, an in-progress upload, or a multi-step flow the server resets on reload;
- anything after a **browser restart or crash** (form data is in memory only, D10; lazy restore starts tabs Discarded; screenshots are purged), or a **renderer crash** (nothing was captured since the last freeze or discard). Capturing edited tabs on input (debounced) would cover renderer crashes; 2b measures its page-channel cost before deciding;
- fields excluded on purpose, fields in cross-origin frames, and fields skipped by the safety rules above (those are offered through the bar where possible).

That is why layer 1 (§7.3) comes first. Sites that restore badly can be marked "don't auto-reload" or "Keep awake".

### 7.6 Measuring memory (`lwb-memprobe`)
- **Metric per OS** (matches what users see in their task manager):
  - Windows: **private working set**, read cheaply via `GetProcessMemoryInfo` + `PROCESS_MEMORY_COUNTERS_EX2.PrivateWorkingSetSize` (no working-set page walks on the hot path). `EX2` needs Windows 10/11 22H2 **with the September 2023 cumulative update**; unpatched machines (likely among old-PC users) fall back to `PROCESS_MEMORY_COUNTERS_EX` private usage or the per-process private working set from `NtQuerySystemInformation`, labelled as such.
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
- **Scriptlet bundle licensing:** popular scriptlet sets are GPL-licensed (uBlock Origin's) or mixed (Brave's adblock-resources includes uBO-derived files). Phase 3a checks the license of whatever bundle we ship with before bundling it, and records it in DECISIONS. Our own scriptlets, written from scratch, are the fallback.

### 8.2 Dark mode
1. **Native preference:** tell pages the user prefers dark (`prefers-color-scheme: dark`). WebView2 `PreferredColorScheme`; WebKit follows the app/window appearance.
2. **Forced dark** for sites without a dark theme: injected CSS filter mode (invert + hue-rotate, re-inverting images and video), per-site toggle, off by default.
3. Browser UI: light, dark, or follow system.

### 8.3 Audio controls
- Per-tab **mute**: native on Windows (`IsMuted`) and Linux (`is-muted`). On macOS, public WKWebView has no per-view mute (verify); use private `_setPageMuted:` under D7, else a JS fallback that mutes media elements (misses Web Audio).
- Per-tab **volume boost** (0–400%) via injected Web Audio (`GainNode`). Cross-origin media without CORS can't be routed: detect it, fall back to 100%, and explain why in a tooltip.

### 8.4 Daily-driver features
**Phase 1 (needed to "browse normally for a day"):** address bar with search (configurable engine), back/forward/reload/stop, **keyboard shortcuts that work even when a page has focus** (native accelerator handling per backend: WebView2 `AcceleratorKeyPressed`, macOS menu key equivalents / `NSEvent` monitor, GTK key handlers). Shortcuts: Ctrl/Cmd + T, W, Shift+T, L, R, F, Tab/Shift+Tab, 1–9, +/−/0, Alt+←/→. Also: **our own page context menu** (incl. "Open link in new tab", Inspect when devtools are enabled), **permission prompts** with per-site storage, **web notifications shown as OS notifications** (a click activates the tab; WebView2 `NotificationReceived`, handled so WebView2's own UI never appears; WebKitGTK `show-notification` (verify); macOS: WKWebView has no Web Notifications API, so an injected `Notification` shim (page-reported, main world) forwards to `UNUserNotificationCenter`), **element fullscreen** (chrome hides, view fills the window), **certificate error interstitial**, **HTTP auth dialog**, **JS dialogs** (alert/confirm/prompt, `beforeunload` on close), **HiDPI and moving between monitors** with different scale factors, **file drag and drop into pages** (e.g. Gmail or Drive uploads must reach the content view; wry's `with_drag_drop_handler` intercepts drops when set), **find in page**, **history recording** (data only; UI in Phase 4a), per-site zoom, downloads list, crashed-tab page with Reload, DevTools (behind a setting).

**Phase 2c:** **tab search, including sleeping (discarded) tabs** (title and URL; archived tabs too once 4b adds them).

**Phase 3b:** an **autoplay setting** (block / allow muted / allow): WKWebView `mediaTypesRequiringUserActionForPlayback`, WebKitGTK `WebKitWebsitePolicies`; WebView2 only has the `--autoplay-policy` browser argument, which must be identical for every view in the environment (§6.4), so it's a restart-to-apply setting there.

**Phase 4a:** history UI, bookmarks (+ bar), address-bar autocomplete, **private windows**, **clear browsing data** (incl. captures and screenshots, §7.5), **multiple windows** + moving tabs between them (never between private and normal windows or across spaces with their captures, §7.5).

**Phase 4b:** **print** and **PDF viewing** (built into WebView2/WKWebView; WebKitGTK (verify), else fall back to opening externally); **default-browser registration** (Windows: installer registers capabilities, then opens the Default Apps page, since apps can't set themselves as default) **with single-instance URL handoff**, so links opened from other apps reach the running browser (Windows: a second instance forwards its command line to the first; macOS: `application:openURLs:` / Apple Events; Linux: a `.desktop` `%u` entry plus a single-instance handoff over D-Bus or a local socket). The handoff is an untrusted-input parser with its own fuzz target, separate from the bench channel. Also **many-tab usability** (overflow with 50+ tabs, close duplicates), **optional auto-archive** of long-unused tabs (off by default; an archived tab leaves the tab strip, keeps its saved state, and stays findable via tab search and history), **Global Privacy Control** header and an **HTTPS-upgrade mode**.

**Phase 4c:** onboarding and importers (§8.9).

**Ongoing baseline from Phase 1:** keyboard focus moves predictably between the chrome and content; chrome HTML is screen-reader labelled; IME input works in the address bar.

### 8.5 Updates
Phase 4d: on startup (at most daily), check the GitHub Releases API and **notify only**. Releases include an **ed25519 (minisign) signature** of each artifact, checked before telling the user an update is genuine. This works without paid OS code signing.

### 8.6 Crash diagnostics
WebView2 writes Crashpad dumps for its own processes. By default it follows the OS consent setting and may **upload** them, so we set the environment option **`IsCustomCrashReportingEnabled = true`** (§6.4), which keeps dumps local in `CrashDumpFolderPath`. For our shell: a panic hook writing a log, plus a "Copy diagnostics" button on `lwb://about`. No automatic upload.

### 8.7 Extensions (Phase 5b)
1. Built-in features first (§8.1–8.3).
2. **Our own lightweight extension format:** manifest + content scripts + CSS + a small, permissioned API (storage, tab info, messaging), mirroring a subset of WebExtensions so simple Chrome extensions port easily. Works on all three OSes.
3. **Windows only, experimental:** load Chrome extensions via WebView2's browser-extension API. Compatibility is partial and password managers are the hard case. Mark it clearly as Windows-only.

### 8.8 AI: one slider, off by default (Phase 6; DEC-010)
The onboarding page (§8.9) and Settings both show a **3-position slider: "No AI" / "On request" / "Full AI"**. **The default is No AI.** Moving it is always the user's voluntary choice.

| Level | What it means |
|---|---|
| **No AI** (default) | No AI code loaded, no AI network requests, and **no AI interface anywhere**: no buttons, menu items, context-menu entries or "try AI" prompts. The only AI-related UI is the AI section in Settings (where the slider lives) and the onboarding slider. Costs zero memory (verified by benchmark). "Loaded" is defined below |
| **On request** | AI runs only when the user explicitly invokes it (a click or shortcut). Nothing runs in the background, and no page content is sent without that action |
| **Full AI** | Adds proactive features |

- **Every AI feature has its own toggle in Settings.** The slider sets their defaults. If the user changes any toggle, the slider shows **"Custom"**.
- **Our public promise, worded exactly: "Updates never raise your AI level, and new AI features stay off unless you chose Full AI."** A new AI feature defaults to on only for users on Full AI, **off for users on Custom**, and is announced in the release notes. This promise goes on the "How we're funded / what we collect" page (Phase 4d).
- **Agents that act on pages** (clicking, typing, submitting, navigating on the user's behalf) are excluded from every level until D9 decides otherwise. Features that only *read* a page on the user's request (summarize, ask about this page) are governed by this slider, not by D9.
- **AI output enters a page only through the user.** A feature that would change page content (rewrite/proofread a field, translate a page) shows its result in our own UI first; it reaches the page only when the user previews it and confirms (that is the user's action, issued as a command, §6.7), or it goes to the clipboard. Translating a page in place is a page-modifying feature with its own toggle.
- **What "loaded" means:** AI code lives in a separate module, either a library loaded on demand (DLL / dylib / `.so` with a stable C ABI) or a separate helper process over IPC, never statically linked into the shell. At No AI it is neither loaded nor started. D11 (bundle vs download) is about how it ships; "not loaded at No AI" holds either way. If bundled, it counts toward the §4 installer budget.
- **Core features never depend on AI:** hibernation, protection, blocking and restore work identically at every level.
- Page content is sent only as the level allows. **Our proxy keeps nothing beyond the request; the model provider's own retention policy applies** (bring-your-own-key providers, and the upstream provider behind our credits, often keep data for days for abuse monitoring), and Settings links to it. Each feature's toggle states what leaves the machine (e.g. tab titles and URLs for tab-tidy suggestions), which matters most for Full AI's proactive features that run without a per-use click.
- **Providers:** bring-your-own key (free) or prepaid **credits** via our proxy on the owner's server. Free on-device OS models and the overall cost model are D13.
- **Packaging (D11):** either bundle the AI module and load it only when the slider is above No AI, or download it on first use. Decide after measuring the size difference; the owner leans towards bundling if the difference is small, because a download on first use is a bad first experience. Either way, No AI costs zero RAM.

**Draft feature list** (the final list is decided in `docs/AI-SPEC.md` before Phase 6):
- *On request:* summarize page; ask about this page (side panel); explain/define a selection; translate a selection or page (shown in our UI; in-place page translation is page-modifying, see above); rewrite/proofread text in a field (previewed, inserted only on confirm); summarize a video from its transcript; plain-language search over tabs and history.
- *Full AI adds:* tab-tidy suggestions (duplicates, stale tabs, grouping); automatic names for tab groups and spaces; a "where was I?" recap when reopening a space; an always-available page assistant; gentle hints (e.g. "long article, summarize it?").

### 8.9 Onboarding (Phase 4c; DEC-011)
- Shown **once**, on first launch. **Entirely optional:** a prominent **"Skip, just browse"** button, and every step has a sensible default, so the user can start browsing immediately after install.
- Steps, each skippable and all changeable later in Settings:
  1. **Import** from Chrome, Brave, Edge, Firefox or **Arc**: bookmarks, history, open tabs/sessions. For Arc, its local sidebar and spaces data (Arc stops working on 22 December 2026, so the importer reads Arc's local files and never needs Arc to run). Arc spaces are imported as **bookmark folders** (one per space) until spaces exist; Phase 5a offers to convert them into spaces. Passwords and cookies are never imported. Every importer is a parser of untrusted files and gets a fuzz target (§6.6).
     - Format notes: Chromium history databases are locked while the browser runs (copy them first); Chromium open tabs live in undocumented SNSS session files; Firefox `sessionstore.jsonlz4` uses mozlz4; Arc's data layout differs between macOS and Windows.
     - **The Arc importer is best effort** and cut first if 4c overruns: Arc stops working before 4c ships, and uninstalling Arc may remove its local files (especially on Windows). It covers Arc for Windows' data location as well as macOS.
  2. **Theme** (light / dark / system).
  3. **Hibernation preset** (Relaxed / Balanced / Aggressive; default per D6).
  4. **Ad blocking** on/off (default on).
  5. **Search engine:** a fair choice list. The default here is tied to search-partnership revenue (§5.2, D4).
  6. **AI slider** (§8.8), starting at No AI.
- Re-openable later from Settings.
- **No account and no email required**, ever, to use the browser.

### 8.10 Spaces (Phase 5a; DEC-012)
- A **space** is a named set of tabs in a window that the user can switch between (e.g. "Work", "Personal"). Tabs in other spaces follow the normal lifecycle (§7.1) and are usually asleep.
- Each space can optionally have **separate cookies and logins**:
  - Windows: a WebView2 profile per space (`ProfileName`, exposed by wry as `with_profile_name`), sharing one environment and browser process (§6.4);
  - macOS: `WKWebsiteDataStore(forIdentifier:)` (exposed by wry as `data_store_identifier`), which needs **macOS 14+** while §3 says 13+ (D12). On macOS 13, a separate-login space could be **ephemeral only** (`nonPersistent()` data store);
  - Linux (webkit2gtk-4.1): a separate `WebKitWebContext` per space, created with its own `WebKitWebsiteDataManager` (`webkit_web_context_new_with_website_data_manager`, separate data and cache directories; wry's `WebContext::new(Some(dir))`). `WebKitNetworkSession` is a WebKitGTK 6.0 API, not available on 4.1. Each context needs its own network process, plus its own web processes, and two contexts must not end up sharing one cookie database (WebKit bugs 239521, 222366).
- Per-space memory costs more on Linux and macOS than on Windows. **Measure the cost of each extra space on all three OSes** when this is built, and report it on `lwb://memory`.
- `ProfileId` (§6.3) is in the engine trait from 1a, so 5a doesn't need a refactor session.
- Offers to convert Arc spaces imported as bookmark folders (§8.9) into spaces.

## 9. Benchmark methodology (`tools/lwb-bench`)
- Launches each browser with a **fresh profile** and a fixed URL list, waits for loads to settle, then samples memory (§7.6 metrics, whole process tree) every 5 s for 5 min. Reports median and p90. 3 runs per configuration; report the median of the medians. Also records page-load times (for the blocker-overhead target).
- **Required browsers/configs:** ours (hibernation off; default preset), **Brave with Memory Saver Off, Moderate, Balanced (the default) and Maximum**, Chrome (same modes, if it still offers all of them), **Edge (Windows) with sleeping tabs off, at its default, and at its most aggressive setting** (the exact settings, e.g. the shortest sleep timeout and any related efficiency option, are fixed in 0d and documented). The bench sets the mode explicitly in each fresh profile, so the owner's personal browser settings don't affect results. Brave, Chrome and Edge also have "always keep these sites active" lists; they stay empty in benchmark profiles, matching our empty-by-default user list except for the shipped defaults, which the results note. **Headline comparisons use Brave Maximum and Edge's most aggressive setting** (DEC-008); the defaults are always reported alongside. **Safari is a required config on macOS** (manual if it can't be automated), because it is the same engine with no extra chrome renderer and any memory claim on macOS invites that comparison. Optional extras: Firefox. Fresh profiles, no extensions, default settings otherwise. Brave's built-in blocking stays on (noted in results).
- **Autoplay:** fresh profiles block unmuted autoplay, so the YouTube tab is started the same way in every browser: a scripted click. If impossible, use `--autoplay-policy=no-user-gesture-required` for all Chromium-based browsers and document it.
- **Tab sets** (`bench/tabs-standard.txt`, `bench/tabs-realistic.txt`, URLs fixed and versioned in Phase 0d):
  - *Standard (20, logged out):* YouTube video (playing), Wikipedia, two news sites, Reddit thread, GitHub repo, Stack Overflow, MDN, Google Maps, shopping product page, recipe site (ad-heavy), Twitch channel, a docs site, a blog, weather, sports, long-form article, image-heavy site, Excalidraw, a search results page.
  - *Realistic mix (20):* the standard set, but 5 tabs must stay alive, each with the §7.3 rule expected to protect it in our browser:
    1. audio playing (rule 1);
    2. a `getUserMedia` test page, a simulated call (rule 2);
    3. a page with a half-filled form (rule 8), **filled with OS-level synthesized input in every browser**, the same method for all, so it counts as a trusted user interaction;
    4. a site with notification permission granted (rule 7);
    5. a chat-style app with a live connection, from a site that is **not on any browser's default keep-awake list**, which the bench adds to each browser's own "always keep active" list the same way a user would (rule 5). This also tests that exception lists actually work.

    **"Alive" means not discarded and not frozen, with JavaScript running**, checked by a heartbeat each test page reports. "Frozen" is reported separately (Edge's sleeping tabs freeze rather than discard). Measured at 30 min. Chrome's Memory Saver already exempts several of these activities (§1), so competitors may keep 4 or 5 alive; the comparison is reported either way.
  - *Extra cases* (reported separately, outside the headline): cases competitors may not cover, such as a form inside an SPA that Chromium might not detect as "partially filled", and a tab the user left under 2 minutes ago (rule 6). This metric shows what we do better than Memory Saver: our browser should use no more memory than Brave Maximum or Edge's most aggressive setting **and** keep all 5 alive, and **the results report how many of the 5 each competitor kept alive**.
  - *Forced-discard form check* (part of the realistic mix, run in every browser): half-fill the form tab, force a discard (ours: the bench control channel's "hibernate now"; competitors: their own discard mechanism, e.g. `chrome://discards` where available, documented per browser), reactivate it, and check whether the field contents came back. Reported per browser.
- **Driving our browser:** a `bench`-feature-only control channel (CLI flags + a local socket bound to 127.0.0.1 with a random token) to open tab sets, fast-forward the policy clock ("30-min idle simulation") and read state. Never in release builds (§6.6).
- **Owner scenario:** a manual checklist reproducing the owner's usual logged-in Brave session, measured in both browsers.
- **Machines:** the owner's main Windows machine, plus an **8 GB RAM Windows 10 machine** where one is available (the old/low-RAM audience, §1). Results name the machine.
- **Energy (optional):** per-process energy over a run on Windows via `powercfg /srumutil`, summed over each browser's process tree. Supporting evidence for battery claims only (§1).
- **Public claims** (from the Phase 2c write-up on): consider recording the tab set with a replay proxy (e.g. Web Page Replay) so runs months apart stay comparable.
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
- memory: empty window total with and without the chrome view;
- **WebView2 telemetry:** confirm what the WebView2 runtime sends to Microsoft (Microsoft documents required diagnostic data that no embedder can turn off, optional data that follows the Windows setting, and embedder control over crash reporting and SmartScreen only; §1) and which settings, policies or environment options affect it. Verify that `IsCustomCrashReportingEnabled = true` keeps dumps local, and record what SmartScreen sends (for D16);
- **DRM video:** whether Netflix, Disney+ and Spotify web play, which DRM system is used (Widevine via the Edge runtime; whether PlayReady is available), and the **maximum resolution** each service reaches;
- **resource limiter groundwork:** whether WebView2's child processes can be placed in a Job Object (CPU rate caps) or given EcoQoS (`SetProcessInformation`), for a possible future limiter. WebView2's browser process is launched as a child of ours, so putting our process in a job before creating the environment should put the whole tree in it: test whole-browser CPU caps and `JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE` (which would also clean up after a `panic = "abort"` crash). Per-renderer caps depend on nested-job rules, because Chromium's sandbox already puts renderers in jobs; also check whether Chromium's own priority management overrides our EcoQoS settings. No promise is made before this is known;
- **suspended views and scripts:** whether `ExecuteScript` resumes a `TrySuspend`ed view (undocumented; §7.5 capture timing);
- **isolated world alternative:** whether a bundled unpacked extension's content scripts can run our page scripts in an isolated world and reach the host (§6.6).
- **Exit:** every ❌ gets a DECISIONS entry (workaround / direct bindings / scope change) and a go/no-go for Windows.

**0c. macOS + Linux engine spike**
The same spike app and checklist, adapted, in `docs/spikes/phase0c-macos.md` and `phase0c-linux.md`. Extras: Linux on **Wayland and X11** via the GTK path; WebKit content rule lists; macOS media/capture state APIs (incl. whether `fullscreenState` is available on macOS 13); macOS mute and process attribution (pick a method under D7); WebKitGTK permission requests, PDF, `camera-/microphone-/display-capture-state`, and whether script messages identify the sending frame (§6.6); **DRM on Linux** (WebKitGTK ships no Widevine CDM, so Netflix, Disney+ and Spotify web are expected not to play; confirm and record); **passkey / security-key sign-in on macOS** (expected to fail without Apple's restricted browser WebAuthn entitlement, §12). Run on GitHub's macOS runners (which have a GUI session) for the automatable checks. Hands-on macOS checks wait for the MacBook. Can run in parallel with 0d.

**0d. Memory probe + benchmark v0 + baselines**
`lwb-memprobe` (Windows implementation first, incl. the pre-September-2023 fallback, §7.6; macOS/Linux stubs following §7.6) and `lwb-bench` v0 (launch other browsers with fresh profiles + tab set, sample, report). Fix the two tab-set files. Produce Windows baselines on the owner's machine (and an 8 GB Windows 10 machine if available) for Brave (Memory Saver off, Balanced, **Maximum**), Chrome (off/on) and Edge (sleeping tabs off, default, **most aggressive**), and fix the exact Edge settings used. **Also run the realistic mix (§9) and the forced-discard form check against Brave and Edge**, reporting how many of the 5 tabs each kept alive (and how many were merely frozen) and whether Chromium's reload restored any form fields. This result comes **before any public copy** is written, and the owner confirms the headline with it in hand (§1). The orchestrator then recalibrates §4.
Also: `bench/README.md` (statistic + method per §4 target), and a CI job that reports installer-independent budgets measurable so far (shell binary size; cold start of the spike or app if available), failing on regression beyond the stated tolerance.
- **Acceptance:** reproducible within ±10% across 3 runs; results committed in `bench/results/`.

### Phase 1: Core browser
**1a. Engine traits + Windows backend + browsing basics:** the `Engine`/`EngineView` traits (§6.3, incl. `ProfileId`) and the Windows backend; the command layer (§6.7), with every 1a action going through it; view kinds and view swapping at the internal/web boundary (§6.6); window with chrome view (tab strip: new/close/switch/reorder/pinned; address bar: URL or search; back/forward/reload/stop; loading indicator; title + favicon); native accelerators; `lwb://newtab`, `lwb://about`; new-window requests → new tab (with opener popups that inherit the opener's setup, §6.6); overlay mechanism (§6.5); HiDPI.

**1b. Storage + session:** `lwb-storage`; fuzz targets for the IPC and page-channel message parsers; settings + `lwb://settings`; incremental session save (debounced) + lazy restore (minimal Discarded state, §7.1); history recording.
- **Acceptance (end of 1b):** **500-tab session restore test** (a generated session of 500 tabs): the browser restores it without hanging, meaning input-to-paint p95 < 100 ms during the restore and the active tab painted within 2 s; the active tab loads first; all 500 tabs are present afterwards. Report the memory used by 500 discarded tabs (tab-strip DOM and favicons in the chrome view).

**1c. Engine-event UIs:** JS dialogs; permission prompts + per-site storage; **web notifications** (§8.4); certificate interstitial; HTTP auth; fullscreen; our page context menu; find in page; downloads; crashed-tab page; per-site zoom; DevTools setting; file drag and drop into pages (§8.4).
- **Acceptance (end of 1c):** the owner browses normally on Windows for a day (YouTube, Gmail with notifications, GitHub, a banking site, Google sign-in popup) with no blocking bugs. Empty-window memory recorded.

**1d. macOS + Linux backends** to feature parity with 1a–1c where `EngineCaps` allow, including the macOS `Notification` shim. Measure WebKitGTK on modest hardware (§12). Acceptance: builds and launches in CI; manual smoke test on Linux (Wayland); macOS "CI-built, not run" until the MacBook arrives.

### Phase 2: Memory management
**2a. Lifecycle, signals, protection:** full state machine with engine events (§7.1); trusted + page-reported signals (§7.2) per backend; all protection rules incl. expiry and the user-interaction requirement (§7.3); Keep-awake; Always-keep-awake list; restore with history and the POST "click to reload" rule. `plan()` unit tests covering: idle timers, every protection rule, page-reported expiry, the podcast scenario, a forged "unsaved input" message without user interaction (must not protect), notification-site rule, previously-active grace, engine-initiated resume, freeze failure, a capture that times out (discard proceeds).

**2b. Page-state capture and restore** (§7.5): edited-field capture, the restore-safety rules and the "view / copy" bar, screenshots via `snapshot()` (debounced capture while visible, encrypted local cache, sensitive-page skip, display in the chrome view), the global and per-tab caps, every exclusion, the in-memory-only rule, and clearing on tab close and *Clear browsing data*.
- **Form-restore test suite** (local test pages served by the test harness):
  - **contents:** plain HTML fields (text, `textarea`, `select`, checkbox/radio); React, Vue and Angular controlled inputs; fields rendered after `load`; a same-origin iframe; dynamically added rows; a rich-text editor (via the bar path); a multi-step form; a long scrolled page;
  - **"comes back"** means the DOM value matches **and** the page's own model saw it (e.g. a React-controlled input whose submit handler reads its state); focused field and scroll position are restored too;
  - **multi-step expected result:** fields of the step the page reloads into are restored; fields of steps the server reset are offered through the bar;
  - **safety cases** (one per §7.5 restore rule): an auto-saving page whose server copy changed (conflict, nothing overwritten), a `select` that submits on change, a settings toggle that saves on change, a login redirect after the session expired (no restore, bar shown), the user typing during a delayed restore (restore stops), a rich-text editor (bar, no DOM write);
  - **exclusion cases:** password fields (incl. one toggled to `type="text"`), `cc-*`, `one-time-code`, `new-password`, hidden and file fields are **not** captured; a cross-origin frame is not captured and suppresses the screenshot; a private window captures nothing; captured data is gone after the tab closes and after *Clear browsing data*; an oversized or malformed capture message is rejected;
  - a separately reported **real-site corpus** (about 20 popular sites with forms), with no target.
- **Acceptance (end of 2b):** 100% of the supported fields in the suite come back (§4), and every safety and exclusion case passes, on Windows; on macOS/Linux where `EngineCaps` allow.

**2c. Budget + dashboard:** memory limit with presets, hysteresis, cooldown and OS pressure triggers (§7.4); event-driven scheduling; `lwb://memory` (total, per-process, per-tab where supported, states, protection reasons, "hibernate now"); tab-strip indicators; learning prompt; **tab search including sleeping tabs** (§8.4). Tests: budget pressure discards (never freezes), the all-protected case, hysteresis, cooldown.
- **Acceptance (end of 2c):** realistic-mix target in §4 met (including the forced-discard form check) or the gap explained; zero wrongly hibernated tabs in the scripted 60-min Aggressive-preset test; first public benchmark write-up.

### Phase 3: Built-in features
**3a. Ad/tracker blocking** (§8.1) on all three backends, list updates, cosmetic filtering, per-site toggle, badge, plus the MV3-extension evaluation on Windows. Acceptance: known-ad test page blocked on all 3 OSes; blocker page-load target (§4) met on Windows; benchmark rerun.

**3b. Dark mode + audio** (§8.2, §8.3) and the autoplay setting (§8.4). Acceptance: forced dark is usable on 10 sample sites; volume boost works on YouTube; mute works on all 3 OSes (macOS via D7 decision).

### Phase 4: Daily-driver polish and tester release
**4a. Browsing data and windows:** everything in the §8.4 Phase 4a list.

**4b. Polish:** everything in the §8.4 Phase 4b list (incl. single-instance URL handoff and optional auto-archive), a command palette listing every command (§6.7), user-editable keyboard shortcuts, final name/icon if decided (D2), About page with donation/sponsor links.

**4c. Onboarding + importers** (§8.9): the onboarding page and its importers (Chrome, Brave, Edge, Firefox, Arc) with their fuzz targets. Acceptance includes: "Skip, just browse" reaches a usable window in one click; every onboarding step can be skipped and changed later in Settings; the AI slider step starts at No AI.

**4d. Release engineering:** CI check of the installer-size budget (§4); packaging with `cargo-packager` (Windows NSIS/MSI unsigned; macOS `.dmg` with an ad-hoc signed `.app`; Linux `.deb` + AppImage); checksums + minisign signatures; update notifier (§8.5); crash diagnostics (§8.6); `docs/TESTERS.md` (§11); in-app "Send feedback" link; a public **"How we're funded / what we collect"** page (revenue sources §5.2, what we collect (nothing) and what the WebView2 runtime sends (§1, per the 0b finding), the exact "updates never raise your AI level" promise (§8.8), how to export your data), linked from `lwb://about` and `TESTERS.md`.
- **Acceptance:** builds install on a clean Windows 11 machine (incl. the Smart App Control note) and on macOS by following `TESTERS.md`; first 5–10 external testers onboarded.

### Phase 5: Spaces and extensions
**5a. Spaces** (§8.10): spaces, optional separate logins per space, conversion of imported Arc folders, per-space memory measured on all three OSes, an opener popup from a non-default space (§6.6). Depends on D12 for macOS.

**5b. Extensions** (§8.7).

### Phase 6: AI and credits (§8.8, after `docs/AI-SPEC.md`, which settles the final feature list, D11 and D13)
Acceptance includes: at No AI, the AI module is not loaded and no AI process runs, no AI network request is made and no AI UI appears outside Settings (verified by benchmark and a UI check); a fresh install and an upgrade both land on No AI; an upgrade adding a new AI feature leaves it off for No AI, On request and Custom users.

### Phase 7: Growth and revenue
Website, `docs/MARKET.md` (starting from the research report `docs/research/2026-11-browser-positioning.md`, spot-checking its quotes on the source pages first; sourced user complaints about browser memory use and about Memory Saver unloading tabs people needed, mapped to our features; also where to find first users, e.g. r/browsers, r/pcmasterrace, Show HN, after checking each community's self-promotion rules), search-partnership outreach, sponsorships, code signing + notarization once tester feedback is positive, auto-update, optional sync.
### Phase 8: Engine research (decision gate)
Evaluate Servo as an alternative backend behind the `Engine` trait, and decide whether owning an engine is worth it. Output: a report and recommendation, not a commitment.

## 11. Distribution before code signing
- **Free signing programmes for open-source projects (e.g. SignPath Foundation) generally require an OSI-approved license.** FSL isn't OSI-approved, so assume we don't qualify. Check again, but plan around paid signing.
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
| Brave/Chrome/Edge already have memory savers (Brave Maximum took the owner from 8.1 to 2.6 GB), and Chrome's documented exemptions already keep tabs with audio/video, calls, screen sharing, notifications, downloads, partly filled forms, pinned tabs and connected devices awake | High | On Windows, parity in the realistic mix is the honest expectation. Compete on what they don't do: restoring form state after a discard, showing why each tab is awake, previously-active grace, learning and visible Keep-awake, a user-set memory limit that never discards protected tabs. Benchmark at their most aggressive setting, including the extra cases (§9) |
| Pages forge "protected" signals | Medium | Trusted vs page-reported split; interaction requirement; expiry (§7.3) |
| macOS process attribution needs private API | Medium | D7 policy; totals-only fallback |
| Linux Wayland child views | Medium | GTK path; tested in 0c |
| Ad-block handler slows page loads on Windows | Medium | Fast handler, latency benchmark, MV3-extension alternative (3a) |
| Hijacked filter list runs scripts | High | Pinned list URLs; scriptlets only from a bundled resource set |
| Hibernating something the user needed | High (trust) | Conservative rules, grace periods, Keep-awake, learning prompt, zero-tolerance metric |
| Unsigned builds / Smart App Control block testers | Medium | `TESTERS.md`, signatures; D8 trigger for cheap signing |
| Ad-blocking arms race (YouTube) | Low/Medium | Best effort, no promises |
| License misunderstanding | Medium | FSL + CLA, lawyer review before commercialisation |
| "Never loses your work" is tested publicly, and one lost form undoes it | High (trust) | Claim rests on the §7.3 protection rules first; the form-restore test suite (2b) and the realistic-mix forced-discard check back it; the honest limits (§7.5) are published with the claim; the headline is confirmed only after the 0d competitor run; "100%" never appears in public copy |
| Restoring form data corrupts or leaks user data (stale overwrite, auto-submit, screenshots of sensitive pages) | High (trust, privacy) | §7.5 restore-safety rules and screenshot rules, each with a 2b suite case |
| Passkeys and security keys fail on macOS builds (WebAuthn in a third-party WKWebView needs Apple's restricted browser public-key-credential entitlement and a provisioning profile, which ad-hoc builds lack) | High on macOS (sign-in) | Test in 0c; request the entitlement when signing is set up (D8 / Phase 7); tell macOS testers in `TESTERS.md` |
| No DRM video on Linux (WebKitGTK ships no Widevine CDM) | Medium on Linux | Confirm in 0c; state it in `TESTERS.md` and the website, not as a surprise in 1d |
| WebKitGTK performs poorly on weak Linux hardware | Medium | Measure on modest hardware in 1d; don't market Linux as "revive an old laptop" |
| WebView2 support on Windows 10 ends (at least Oct 2028) | Medium | Windows 10 is an audience, not a dependency; track Microsoft's dates; plan the messaging before the end date |
| Maintaining WebKit off Apple platforms is costly (Kagi stopped Orion for Windows/Linux, citing it) | Medium | We use the OS's own engine and never port WebKit ourselves; Linux stays on the distro's WebKitGTK |

## 13. Open decisions and questions (owner)
| # | Decision | Recommendation | Status |
|---|---|---|---|
| D1 | License | FSL-1.1-ALv2 + CLA | **Decided** (DEC-001) |
| D2 | Product name | TBD; check trademark/domain availability | Open (needed by Phase 4b) |
| D3 | Repo visibility | Public | **Decided** (DEC-002) |
| D4 | Default search engine | Configurable; pick the default when a partnership exists | Open (Phase 4) |
| D5 | Overlay mechanism | Grow the chrome view (§6.5) unless 0b says otherwise | Pending 0b |
| D6 | Default hibernation preset | Balanced (5 min freeze / 30 min discard) | Open |
| D7 | Private Apple APIs (mute, process attribution) | Allowed behind the `macos-spi` feature + runtime check + public fallback | **Decided** (DEC-004) |
| D8 | Earlier cheap Windows signing | Only if Smart App Control blocks testers (§11). Signing also gates the macOS passkey entitlement (§12) | Open (Phase 4d) |
| D9 | Automation of the browser or pages: outside surfaces (CLI control, socket API, MCP) **and in-browser agents that act (click, type, submit, navigate) on the user's behalf**. AI features that only read a page on request are governed by the AI slider (§8.8), not D9. The `bench`-feature control channel is a dev-only tool outside D9 | Not before Phase 6; opt-in only, local-only, per-session approval | Open (scope widened in DEC-010) |
| D10 | Should captured form data (§7.5) survive a browser restart? | Not by default. If yes, encrypt with DPAPI (Windows) / Keychain (macOS) / libsecret (Linux), never in private windows | Open (Phase 2b) |
| D11 | AI module packaging: bundled and loaded only above No AI, or downloaded on first use | Bundle if the size difference is small (a download on first use is a bad first experience); measure first. No AI costs zero RAM either way | Open (decided in `docs/AI-SPEC.md`) |
| D12 | Separate-login spaces need macOS 14+ (`WKWebsiteDataStore` identifiers); §3 says macOS 13+ | Raise the macOS minimum to 14; make separate-login spaces macOS-14-only; or offer **ephemeral-only** separate-login spaces on macOS 13 (`nonPersistent()` data store) and persistent ones from 14 | Open (before Phase 5a) |
| D13 | AI cost model | Free AI via on-device OS models (Apple Foundation Models on macOS; Windows on-device APIs on supported PCs) and bring-your-own-key; prepaid credits via our proxy for everything else and for heavy features; no bundled local model by default (memory headline); a free cloud allowance only once there's revenue to fund it. Facts to check (verify): Apple's Foundation Models framework needs macOS 26 on an Apple Intelligence-capable Mac and is Swift-only (needs a small Swift shim; objc2 can't bind it); Windows on-device models (e.g. Phi Silica) run only on Copilot+ PCs and may need package identity or a limited-access unlock. On-device models run in system processes, so `lwb://memory` says their memory isn't attributed to us | Open (decided in `docs/AI-SPEC.md` before Phase 6) |
| D14 | AI levels: one 3-position slider, default No AI, per-feature toggles, updates never turn AI on (§8.8) | — | **Decided** (DEC-010) |
| D15 | Onboarding: optional first-launch page, "Skip, just browse", no account (§8.9) | — | **Decided** (DEC-011) |
| D16 | SmartScreen in WebView2 on or off by default (phishing/malware protection vs sending visited URLs to Microsoft for reputation checks) | Decide after 0b records what it sends; whichever default, show it in onboarding's privacy notes and on the "what we collect" page | Open (Phase 0b finding, decided before 4c) |
| Q1 | Was Brave's Memory Saver on in the owner's baseline? | — | **Answered:** off; 8.1 GB / 64 processes (2.6 GB on Maximum) |

## 14. Workflow (orchestration)
1. **Spec review:** fresh session(s) review this spec; findings go in `docs/reviews/`. The orchestrator folds them in, and the owner approves → status `APPROVED v1.0`.
2. **Per sub-phase:** the owner starts a local implementation session (ideally in plan mode) with `docs/prompts/phase-<id>.md`. The session first follows `docs/prompts/_kickoff.md`: an orientation check of the code it builds on, then a plan and clarifying questions, and it waits for the owner's go-ahead (questions the owner can't answer go to the orchestrator). The prompt is written by the orchestrator, normally once the previous sub-phase is merged. The implementer works on `phase-<id>-<slug>`, keeps `docs/STATUS.md` updated, records deviations in `docs/DECISIONS.md`, and opens a PR.
3. **Review:** a fresh review session checks the PR against this spec's acceptance criteria and §6.6 security rules (prompt: `docs/prompts/review-pr.md`). The orchestrator does a final pass; the owner merges.
   - **How to start it:** once the PR is open and CI is green, the owner may ask the implementation session to **launch the review as a separate Claude Code session** (the desktop app's "start a session" feature). It must use **fresh context** (not a fork of the implementer's conversation), its **own git worktree**, and the `review-pr.md` prompt with the PR number and phase id. Starting a session by hand with the same prompt is equally fine. Either way the reviewer sees only the repo and the PR, never the implementer's reasoning, and the implementer never writes or edits the review.
4. If a session's context gets long, it stops at a clean point and writes a handoff note in `docs/STATUS.md`.
