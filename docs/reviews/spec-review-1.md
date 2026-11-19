# Spec review 1 — SPEC.md v0.1

**Verdict: approve with changes.** The direction is sound and the "prove it in Phase 0" structure is right. But three design problems should be fixed in the spec before Phase 0 starts, because they shape the architecture: privilege is attached per *webview* while tabs move between internal and web pages; the HTML chrome can't draw over content webviews; and the injected-script design conflicts with the isolated-world rule. One memory target is also very likely unreachable on Windows as written.

API claims below come from memory of the WebView2 / WebKit / wry docs and should still be confirmed in the Phase 0 spike. Items marked **(verify)** are the ones I'm least sure of.

---

## Blocking

### B1. Privilege is per webview, but tabs go between `lwb://` and the web (§6.5, §6.6)
- **Problem:** §6.6 gives IPC to "the chrome webview and `lwb://` pages" and none to content webviews. But a tab shows `lwb://newtab`, then the user types a URL and the *same* view loads `https://example.com`. IPC handlers (wry's `ipc_handler`, WebView2 `WebMessageReceived`, WebKit script message handlers) are attached to the view, not the page. So either newtab has no IPC, or every website opened from newtab inherits it.
- **Also:** on Windows, wry serves custom protocols as `http://<scheme>.localhost/` (or `https://`), not as a real `lwb://` scheme **(verify on the current wry)**. "`lwb://` must not be navigable from web content" then really means "don't register the protocol on content views, and block `lwb.localhost` in content views", because otherwise a request to that host goes to the real localhost.
- **Change:** add a rule: *crossing the internal/web boundary swaps the tab's view.* Internal pages get their own privileged view type with the protocol and IPC registered; web pages get a view with neither. Also check the sender's origin and frame on every IPC message (the message source URI is available on all three backends) as a second layer. Block any navigation in the chrome webview to anything other than its own pages.

### B2. HTML chrome can't draw over content webviews (§6.5)
- **Problem:** the chrome webview is a strip docked at the top, and content views are separate native child views below it. Anything the chrome draws in HTML (address-bar autocomplete, menus, the tab context menu, permission prompts, download bubble, find bar, the "over memory limit" indicator, tab hover cards) is clipped to the strip. This is the classic "airspace" problem with multiple webviews.
- **Change:** decide this in the spec and test it in the Phase 0 spike. Options: (a) native menus (the `muda` crate, already in the tao/wry ecosystem) for context and app menus, plus temporarily growing the chrome view over the content with a transparent background for dropdowns; (b) a second transparent "overlay" webview, which costs memory and needs measuring; (c) WebView2 visual hosting (composition controller) on Windows. Add "dropdown overlapping content" as a ✅/❌ row in the spike table.

### B3. Isolated worlds can't do what the JS fallbacks need (§6.6, §7.2)
- **Problem:** scripts in an isolated world (`WKContentWorld`, WebKitGTK script worlds) share the DOM but not the page's JS globals. They *can* listen for `play`/`pause`/`input` events. They *can't* wrap `getUserMedia`, see `AudioContext` instances the page created, or see a `beforeunload` handler the page added. Several §7.2 fallbacks need a main-world shim, which the page can tamper with. On WebView2 there's no public isolated-world API for host scripts **(verify)**, and `chrome.webview.postMessage` can be called by *any* script in the page, ads included.
- **Consequence:** every content-originated signal can be forged. Spoofed messages only affect that tab (good), but a page can mark *itself* protected forever ("unsaved input", "playing audio") and opt out of hibernation. Ad-heavy pages would do this by accident just as often as on purpose.
- **Change:** split signals into **trusted** (native engine APIs, our own UI) and **page-reported** (all JS). Page-reported protection should (1) only count after a real user interaction in that tab, (2) expire unless refreshed (for example dirty-form protection lasts until 24 h or until the tab has been idle N hours, configurable), and (3) be shown on `lwb://memory` as "page says…". Say per signal which world its script runs in.

### B4. Windows "hibernation off ≤ 85% of Brave" is unlikely (§4)
- **Problem:** the target assumes savings "from the shared OS engine and built-in blocking". On Windows neither applies against Brave. WebView2 runs its own process tree (it does not share processes with a running Edge, and code pages don't count toward private working set). And Brave already blocks ads by default. So with hibernation off we run the same Chromium with the same site isolation, and parity (~95–105%) is the honest expectation. The macOS target is different: WebKit vs Chromium is a real gain there.
- **Change:** Windows hibernation-off target → "≤ 100% of Brave, ≤ 85% of Chrome" (vs Chrome, blocking does count). Keep 85% of Brave as a macOS target. See S10 for the "hibernation on" target being too *easy*.

---

## Should fix

### Technical accuracy

**S1. Freezing under budget pressure doesn't free memory (§7.4 step 3).** `TrySuspend` releases only part of a tab's memory. With "freeze if possible, else discard" plus a 10 s cooldown, a browser over its limit would freeze every eligible tab one by one (minutes) before discarding any. Change: under budget pressure, **discard**. Freezing is for idle timers only. Add hysteresis (act above the limit, stop below ~90% of it) so it doesn't oscillate.

**S2. Native mute on macOS (§8.3, §7.2).** As far as I know WKWebView has no public per-view mute (`_setPageMuted:` is private SPI) **(verify)**. WebKitGTK does have one (`webkit_web_view_set_is_muted`). Decide the policy on private Apple SPI now, because memory attribution (S4) probably needs it too. We don't ship through the App Store, so it's allowed but fragile. Without it, fall back to muting media elements by script (this misses Web Audio).

**S3. WebView2 capture state (§7.2).** I'm not aware of a public "is this webview capturing mic/camera" property. There is `PermissionRequested` (start hint only) and `ScreenCaptureStarting` for `getDisplayMedia` **(verify)**. "Capture ended" needs the JS `MediaStreamTrack` `ended` tracking, so per B3 it's page-reported. Mark the cell as such rather than "verify".

**S4. macOS process attribution (§7.6).** There's no public API to get a WKWebView's WebContent PID or the responsible PID of an XPC service. Realistic options are private (`_webProcessIdentifier`, `responsibility_get_pid_responsible_for_pid`) or a heuristic (enumerate `com.apple.WebKit.*` processes for our user, matched by launch time). Write this down as a known limitation now. Phase 0 is only picking which compromise.

**S5. Linux child webviews on Wayland (§6.1, §3).** wry's `build_as_child` with a raw window handle works on X11 only **(verify)**. Ubuntu 22.04/24.04 default to Wayland. The Linux backend should use wry's GTK path (`build_gtk` into a `gtk::Fixed`/overlay). Add Wayland to the spike checklist.

**S6. Ad blocking via `WebResourceRequested` (§8.1).** (a) Requests from service workers and shared workers are only intercepted if you register the filter with request-source kinds (`AddWebResourceRequestedFilterWithRequestSourceKinds`), so the spec should require it **(verify the min runtime)**. (b) Every request is marshalled to the host's UI thread, the same thread as the tao event loop, so heavy pages will add latency. Benchmark page-load time, not just memory. (c) Worth evaluating in Phase 0 as an alternative: load a declarativeNetRequest (MV3) blocker as a WebView2 browser extension, so blocking runs inside Chromium with no per-request round trip. It's GPL, but as a separately loaded component that's mere aggregation; check with the lawyer.

**S7. adblock-rust's own memory counts against the 60 MB shell budget (§4, §8.1).** On Windows the engine lives in our process for the whole session. Measure it in Phase 0 with EasyList + EasyPrivacy. Recent adblock-rust versions are much smaller, but it isn't free. On macOS/Linux the engine is only needed to *compile* the content-rule lists and answer cosmetic queries. Compile in a short-lived worker and drop the engine afterwards. Precompute generic cosmetic CSS so the engine needn't stay resident.

**S8. Filter lists can inject script (§8.1).** If scriptlet injection (`+js(...)` / redirect resources) is enabled, a compromised or hijacked list source runs JavaScript on every site. Ship scriptlets only from a pinned, trusted resource bundle. Treat list updates as untrusted input, enforce HTTPS, and pin the list URLs in code.

**S9. Benchmark autoplay (§9).** In a fresh Chromium profile, unmuted autoplay is blocked without a user gesture, so the "YouTube playing" tab won't play in an automated run. Pick one method and apply it to every browser: either a scripted click or `--autoplay-policy=no-user-gesture-required`. Also note that other apps (Teams, Outlook, widgets) run their own `msedgewebview2.exe` trees. Attribute processes by tree / `GetProcessExtendedInfos`, never by process name.

### Targets and positioning

**S10. "Hibernation on ≤ 35% of Brave" is too easy, and Brave/Chrome already discard tabs (§4, §12).** After 30 min idle with Balanced settings, 17 of 20 tabs are discarded by definition, so the number mostly measures the timer. Brave and Chrome already ship Memory Saver (tab discarding), so the §12 risk "Brave/Chrome add similar features" has already happened. Changes: (1) benchmark against Brave **with Memory Saver on** as well as off; (2) add a metric that measures *what we're actually better at*: memory at 30 min with a realistic mix where 5 tabs must stay alive (audio, a call, a form, a chat app); (3) confirm whether the owner's 6.5 GB Brave baseline had Memory Saver on.

### Gaps (daily-driver features missing from Phases 1–4)

**S11. Keyboard shortcuts when the page has focus (§8.4).** When focus is in a content webview, it gets the keystrokes, so Ctrl+T/L/W/Tab would do nothing. You need native accelerator handling per backend (WebView2 `AcceleratorKeyPressed`, an `NSEvent` monitor or menu key equivalents on macOS, GTK key handlers). Put it in Phase 1 and the spike.

**S12. Missing from Phase 1 (each one blocks "browse normally for a day"):**
- **Right-click menu** in pages with "Open link in new tab". Without our own handler it opens through the engine's default menu, bypassing our tab model (WebView2 `ContextMenuRequested`, WebKit equivalents).
- **Permission prompts and per-site storage** (camera, mic, location, notifications, clipboard). WebView2 shows default prompts if unhandled. WebKitGTK **denies** by default unless `permission-request` is handled. Store decisions in `site_settings`.
- **Element fullscreen** (video). The content view must fill the window, the chrome hides, and the window goes fullscreen (WebView2 `ContainsFullScreenElementChanged`).
- **Certificate errors and HTTP Basic auth** dialogs (WebView2 `ServerCertificateErrorDetected`, `BasicAuthenticationRequested`; WebKit `authenticate` / TLS error signals).
- **JS dialogs** (`alert`/`confirm`/`prompt`/`beforeunload` on close).
- **HiDPI and moving between monitors** with different scale factors (child view bounds in physical vs logical pixels).
- **Find in page.** Move it from Phase 3: all three engines have a find API, so it's cheap.
- **History recording** (just the table writes; the UI stays in Phase 4) so autocomplete has data when it lands.

**S13. Missing from Phases 1–4 entirely:**
- **Private window.** Expected from a browser sold on privacy, and each engine supports it cheaply (WebView2 `IsInPrivateModeEnabled`, `WKWebsiteDataStore.nonPersistent()`, WebKitGTK ephemeral context).
- **Clear browsing data** (WebView2 `ClearBrowsingData`, `WKWebsiteDataStore.removeData`).
- **Multiple windows** and moving tabs between them (Ctrl+N; reparenting the webview).
- **Print** (WebView2 `ShowPrintUI`, `NSPrintOperation`, `webkit_print_operation`) and **PDF viewing**: WebView2 and WKWebView have it built in; check WebKitGTK **(verify)**.
- **DevTools / Inspect** (dev setting).
- **Default-browser registration.** On Windows that's installer registry entries plus opening the Default Apps settings page; Windows doesn't let apps set themselves as default.
- **Many-tab usability** for the target audience: tab search, tab-strip overflow with 50+ tabs, duplicate-tab cleanup.
- **Accessibility and IME.** At minimum, keyboard focus moves predictably between the chrome and content views, and the chrome HTML is screen-reader labelled.

### Hibernation safety (§7.3)

**S14. Cases where a tab would be wrongly hibernated or wrongly restored:**
1. **Granted notification permission** (Gmail, Calendar, Slack). Freezing or discarding silently kills reminders. Protect from discard by default, or at least warn.
2. **Live connections with no audio**: chat, trading dashboards, web terminals, Jupyter running a cell, long CI log pages. The keep-awake list helps. Also add learning: if the user reopens a discarded tab within ~60 s twice, offer "Keep this site awake".
3. **Restoring POST results and one-time URLs.** Reloading the saved URL can resubmit a form, or replay a one-time link (email confirmation, logout, payment return URL). Rule: if the last navigation was a POST, or the user changes it per site, a discarded tab restores to a placeholder ("Click to reload"), never an automatic reload.
4. **An open JS dialog, file picker, auth prompt or WebAuthn ceremony** in the tab → protect.
5. **Muted autoplay.** A muted ad video shouldn't keep a tab protected for 5 minutes on repeat. Protect only if *audible* and not muted by the user. Chromium's "playing audio" already means audible; make the user-muted case explicit.
6. **"Active upload"** has no signal source in §7.2. Either define one (page-reported fetch/XHR with a body above X MB) or drop rule 5's upload half.
7. **Dirty-form rule scope.** Typing in a site's search box shouldn't protect a tab forever. Define "dirty" as: a non-default value in a field that isn't a search box, not submitted, and still present, plus the expiry from B3.

**S15. Engine-initiated resume (§7.1).** WebView2 resumes a suspended webview by itself on script execution, navigation or becoming visible, and `TrySuspend` can fail. The state machine needs `Resumed` / `SuspendFailed` events, not just commands.

### Phase sizing (§10)

**S16. Phases 0 and 1 are too big for 1–3 sessions each. Suggested split:**
- **0a** Repo, workspace skeleton, CI, cargo-deny, license (after D1). Small.
- **0b** Windows engine spike (the primary platform, and the go/no-go).
- **0c** macOS + Linux spike. The macOS half is blocked until the MacBook arrives. GitHub's macOS runners have a GUI session, so the automatable checks can run in CI first.
- **0d** `lwb-memprobe` + `lwb-bench` v0 + Windows baselines.
- **1a** `Engine` traits + Windows backend + tabs / address bar / navigation / shortcuts.
- **1b** Storage, settings, session + lazy restore, downloads, crash page, permissions, context menu.
- **1c** macOS + Linux backends to "builds and launches".
- **2a** Signals + lifecycle + protection rules + `plan()` tests. **2b** Budget, pressure triggers, `lwb://memory`, tab indicators.
- **3a** Blocker (it's big on its own: two backends, list updates, cosmetic filtering). **3b** Dark mode, audio.
- Phase 4 is fine but long; the release-engineering half (packaging, updater, minisign) can be its own session.

### Licensing / distribution (factual)

**S17. CI on a private repo (§10 Phase 0, D3).** The repo is currently **private**. Private repos still get free Actions minutes on the Free plan (2,000/month), but macOS minutes count 10× and Windows 2×, so a 3-OS matrix would burn through them in days. Either decide D3 (public) before Phase 0 CI, or limit macOS CI to `main` merges. Also, publishing the repo with no license file means "all rights reserved", which is fine but not the goal, so D1 should be settled before D3.

**S18. Smart App Control vs unsigned builds (§11, Phase 4 acceptance).** On *clean* Windows 11 installs, Smart App Control starts in evaluation mode and can block unsigned apps outright, with no "Run anyway". Turning it off can't be undone without reinstalling Windows. This makes "installs on clean Windows 11 by following TESTERS.md" fail for some testers. Consider cheap signing earlier: Microsoft's Trusted Signing service costs around $10/month for eligible individuals **(verify current eligibility and price)**. At minimum, document SAC in `TESTERS.md`.

**S19. Filter list licenses (§8.1).** EasyList/EasyPrivacy are dual GPLv3 / CC BY-SA 3.0+, and uBlock Origin's lists are GPLv3. Downloading them at runtime (not bundling) keeps this simple. The spec's "if license-compatible; check" can say this outright.

---

## Nice to have

- **N1. Event-driven policy instead of a fixed 5 s tick (§7.4).** Compute the next deadline (the next tab to cross `T_freeze`/`T_discard`) and sleep until then or until a signal/pressure event. Only sample memory periodically when a budget is set. Saves wake-ups and battery, especially on macOS.
- **N2. Cheap Windows memory reads (§7.6).** `GetProcessMemoryInfo` with `PROCESS_MEMORY_COUNTERS_EX2` gives `PrivateWorkingSetSize` cheaply. Avoid `QueryWorkingSetEx` page walks on every tick.
- **N3. WebView2 environment options must match exactly (§6.4).** Webviews share the browser process only if they use the same user-data folder *and* identical environment options, so one mismatched `AdditionalBrowserArguments` means a second browser process (or an error). Create one environment and pass it everywhere.
- **N4. Session crash safety (§6.7).** Save the session on every change (debounced, SQLite WAL), not only on exit.
- **N5. Downloads (§6.6).** On macOS, set the `com.apple.quarantine` xattr on downloaded files ourselves if WebKit doesn't. On Windows, confirm WebView2 writes Mark-of-the-Web.
- **N6. Global Privacy Control header and HTTPS-upgrade mode.** Cheap privacy wins that fit G5.
- **N7. Benchmark reproducibility (§9).** Live pages change daily. For the public claims, consider recording the tab set with a replay proxy (for example Chromium's Web Page Replay) so runs months apart are comparable.
- **N8. Bench control channel (§9).** "30-min idle simulation" needs a way for `lwb-bench` to drive our browser (a CLI flag or a debug-only local socket). Spec it, and make sure it's compiled out of, or disabled in, release builds and never reachable from web content.

---

## Proposed removals / simplifications

| # | Section | Remove / simplify | Why |
|---|---|---|---|
| R1 | §6.1 | "Settings as TOML or SQLite (implementer's choice)" → **SQLite** | SQLite is already required. One fewer decision and dependency. |
| R2 | §6.1 | "`include_dir`/`rust-embed`" → pick one (`rust-embed`) | Avoids two sessions picking differently. |
| R3 | §6.5 | `lwb://hibernated` page | A discarded tab has no view. Loading a page to show "sleeping" would create the webview hibernation just freed. Draw the placeholder in the chrome or natively, and only use it for the "click to reload" case (S14.3). |
| R4 | §6.3 | `set_memory_target(low)` + `suspend()` as separate WebView2-shaped methods → one `set_lifecycle_hint(Active / Background / Frozen)` | Keeps WebView2 vocabulary out of the trait. Each backend maps the hint. `SuspendResult::Unsupported` duplicates `EngineCaps.suspend`. |
| R5 | §6.3 | `snapshot()`, `EngineCaps.chrome_extensions`, `per_view_process_attribution` | Not needed until Phase 2 stretch / Phase 5. Add them when used. |
| R6 | §7.3 rule 9 | Chrome webview protection | The chrome webview isn't a tab, so the policy never considers it. For `lwb://` tabs, reuse rule 3 (dirty input). |
| R7 | Phase 4 | Minidump crash reports | WebView2 already writes crash dumps for its own processes (Crashpad). For our shell, a panic hook writing a log + "copy diagnostics" button is enough for testers and avoids a native minidump dependency. |
| R8 | §9 | Safari and Firefox as required bench targets | They aren't in any §4 metric. Make them optional extras. |
| R9 | §2 G6 | "(Servo, or our own)" → "(e.g. Servo)" | "Our own" is already a non-goal and Phase 8 is the gate for it. |
| R10 | STATUS.md | Phase 0 "Blocked on … D1" | Only 0a (license file/CLA) needs D1. Spikes can start once the spec is approved. |

## Notes on CLAUDE.md
- Once code exists, add the actual build/run commands per OS and how to run the spike and bench.
- Add: "`lwb-engine` must not leak wry or platform types in its public API" (implied by §6.3, worth stating).
- Add: "no private Apple SPI without a DECISIONS entry" (see S2/S4).
- Note that macOS can't be tested by hand yet, so macOS work must be marked "CI-built, not run" in STATUS.md.
