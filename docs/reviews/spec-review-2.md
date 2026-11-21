# Spec review 2 — SPEC.md v0.3 (PR #5)

**Verdict: approve with changes.** v0.3 has the right shape: protection first and restore second, honest limits, a benchmark against competitors at their strictest settings, and an AI slider that really does default to off. Four things should be fixed before the spec is approved, because they affect the headline claim or user data:

1. The positioning leaves out that Chrome/Brave Memory Saver already exempts in-use tabs.
2. The restore algorithm can overwrite or submit user data.
3. Screenshots leak exactly what the form-field exclusions are meant to protect.
4. The realistic-mix benchmark doesn't define "alive", and one of its five tabs has no protection rule.

Everything else is accuracy fixes, gaps and phase splits.

Scope: the `git diff main...spec-v0.3` changes, checked against the rest of SPEC.md, DECISIONS.md (DEC-008 to DEC-013) and the research report. Findings already raised and addressed in `spec-review-1.md` are not repeated. Sources are at the end. **(from memory)** marks claims I couldn't confirm against current documentation during this review. They should go on the 0b/0c checklists as **(verify)**.

---

## Blocking

### B1. Chrome/Brave Memory Saver already protects in-use tabs; §1, §4 and DEC-008 present this as our difference (§1, §2 G3, §4, §9, §12, DEC-008)
- **Problem:** §1 says "every shipping tab discarder treats a discarded tab as reload from the URL" and makes our difference "tabs you're using are never put to sleep". But Google's own help page for Chrome performance settings lists activities that **prevent** Memory Saver from deactivating a tab: audio or video playback and calls, screen sharing, page notifications, active downloads, **partially filled forms**, pinned tabs and connected USB/Bluetooth devices. Brave inherits this. So four of the five realistic-mix tabs (audio, `getUserMedia` call, half-filled form, notification site) may well stay alive in Brave Maximum too. The "how many of the 5 each competitor kept alive" result could then be 4/5 or 5/5. The research report never checked Chrome's exemption list, and that list undercuts its "top finding".
- **Why it matters:** if Brave keeps the same tabs alive, the realistic-mix memory comparison becomes "same Chromium doing the same work". Parity is then the honest expectation on Windows (spec-review-1 B4). Layer 1 becomes table stakes, not the differentiator. What competitors *don't* do is still real:
  - restoring form state and scroll position after a discard that does happen (the forced-discard check);
  - showing *why* each tab is awake;
  - "previously active" grace, learning and Keep-awake that the user can see;
  - honouring a user-set memory limit without discarding protected tabs.
- **Also unverified:** whether Chromium's discard-and-reload restores some form fields or the scroll position from the saved navigation entry, the way back/forward does for simple forms **(from memory)**. The forced-discard check in §9 settles this empirically. Until it has run, "no browser restores your form" shouldn't appear even in internal positioning text.
- **Change:**
  1. §1: replace "Our difference: … tabs you're using are never put to sleep" with wording that admits the overlap. For example: "Like Memory Saver, we don't sleep tabs you're using, but we show you why each tab is awake, and when a tab is slept anyway it comes back with your typed text and position."
  2. §12: rewrite the Brave/Chrome risk row so it names Chrome's documented exemptions.
  3. §9 realistic mix: replace or add must-stay-alive cases that test what competitors *don't* cover, for example a live-connection chat or dashboard with **no** notification permission, a form inside an SPA that Chromium doesn't detect as "partially filled", and a tab the user left less than 2 minutes ago.
  4. Make running the realistic mix and the forced-discard check against Brave/Edge an explicit **0d deliverable**, before any public copy is written. The headline (DEC-008) is the owner's call, but it should be made with that result in hand.

### B2. Restoring can overwrite newer data, trigger auto-save or auto-submit, and fill the wrong fields (§7.5 "Restoring")
- **Problem:** §7.5 restores "field values … matched by the stable selectors, firing `input`/`change` events so the page's own scripts notice". As written, that corrupts data in common cases:
  - **Stale overwrite:** an auto-saving page (CMS editor, issue tracker, webmail draft, a document saved from another device) reloads with *newer* server content. We then write the older captured value over it and fire `input`, and the page auto-saves the old text over the new.
  - **Side effects on `change`:** settings pages that save on toggle, `<select onchange="form.submit()">` filters, cart quantity fields, quiz answers. Restoring a checkbox or select fires a real request that changes account state or submits.
  - **Wrong target:** after reload the server may show a different page: step 1 of a multi-step flow, a login page after the session expired, or a redirect within the same origin. Selectors like `name`/position match fields that mean something else (`name="email"`, `name="q"`, `address`).
  - **Clobbering new input:** SPAs render fields after `load`. A delayed restore can overwrite what the user started typing in the meantime.
  - **Rich-text editors** (ProseMirror, Lexical, Slate, CKEditor, Gmail compose) keep their own model. Writing text or HTML into their `contenteditable` DOM desyncs the editor, and it may then save garbage or throw the text away.
- **Also ambiguous:** "Capture form field values" doesn't say *which* fields. Capturing every field, rather than only the ones the user edited, makes all of the above worse.
- **Change:** add explicit restore-safety rules to §7.5 and test each one in the 2a suite:
  1. Capture only fields the **user edited** after a trusted interaction whose value differs from `defaultValue`/`defaultChecked`/`defaultSelected`, the same definition as §7.3 rule 8.
  2. Restore only if the reloaded page's **URL matches** the captured one (normalized; origin alone isn't enough) **and** the form's field-set fingerprint matches. Otherwise don't restore; show a non-modal "We saved text you typed here: view / copy" bar instead.
  3. Write a field only if its current value still equals its post-load default. If the page loaded different content, it's a conflict: skip and offer the "view / copy" bar, never overwrite.
  4. Fire `input` only for text-like fields. **Never fire `change` on selects, checkboxes or radios automatically**, and restore those only if no `onchange`-driven navigation or fetch is detectable, or restore them via the bar. At minimum, never restore into a form whose `change` triggers navigation.
  5. `contenteditable`: don't write into the DOM by default. Offer the text through the bar (copy / insert), or insert it with `document.execCommand('insertText')` after the user focuses the field, which editors treat as user input.
  6. Stop restoring as soon as the user types or clicks in the tab, and give up after a fixed timeout (e.g. 10 s) waiting for SPA fields.
  7. Add 2a suite cases for each rule: an auto-saving page whose server copy changed, a select that submits on change, a settings toggle that saves, a login redirect, a rich-text editor, and a React-controlled input.

### B3. Screenshots bypass the form-data exclusions and sit unencrypted on disk (§7.5 "Never captured", "Storage"; §6.8)
- **Problem:** the exclusion list protects password, `cc-*`, OTP and new-password fields and leaves out cross-origin frames "(this also leaves out embedded payment forms)". The **screenshot captures the pixels of all of them**: a typed card number in a Stripe/Adyen iframe, an OTP, an IBAN, a bank balance, a medical record, an email body. Those images are then written in plain form to a disk cache that:
  - survives a crash (it is only "cleared at startup", which may be days later);
  - isn't securely erasable on SSDs;
  - on Windows may sit under **Roaming** AppData if it follows §6.8's "app data dir", since the `directories` crate's `data_dir` is Roaming. In managed environments that folder syncs to a server.
- **Also missing:** Clear browsing data and spaces. §7.5 doesn't say that clearing data (or one site's data) deletes captures and screenshots. With spaces (5a), captures must not cross a space boundary.
- **Change:**
  1. Encrypt screenshot files with a **random per-run key held only in memory** (AES-GCM or ChaCha20-Poly1305). After a crash the files can't be read, and the startup purge only tidies up.
  2. Store them under the OS **local cache dir** (`directories::ProjectDirs::cache_dir`: Local AppData on Windows, `~/Library/Caches`, `$XDG_CACHE_HOME`), with user-only permissions.
  3. **Don't take a screenshot** when the page has a non-empty field from the never-captured list, or when focus or input went into a cross-origin frame since load, and on sites in a small, editable "sensitive" list (banking, health; default empty or a short list). Show a plain placeholder instead.
  4. Downscale and compress (JPEG/WebP, about viewport size or smaller), and give the cache a size limit with LRU eviction. 500 full-resolution PNGs on a 4K screen run to gigabytes.
  5. State that *Clear browsing data* and per-site clearing purge captured form data and screenshots, and that captures are keyed by (profile/space, tab) and never restored across spaces or between private and normal windows.
  6. Phrase the privacy promise as "not written to disk by us". Process memory can still reach the pagefile or hibernation file.

### B4. The realistic-mix headline doesn't define "alive", and its chat tab has no protection rule (§4, §9, §7.3)
- **Problem:**
  - The realistic mix requires "all 5 must-stay-alive tabs alive", but "alive" is undefined. Edge's sleeping tabs *freeze* rather than discard. If frozen counts as alive, Edge passes while its chat tab is dead. If it doesn't, our Balanced preset freezes unprotected tabs at 5 min.
  - The "chat-style app (live connection)" tab matches no §7.3 rule. It has no audio, no capture, and no notification permission (that's the fifth tab). It stays alive only if its site is on the shipped Always-keep-awake list (e.g. `discord.com`). If the bench picks a chat site that happens to be on our default list, the result reflects our choice of test site rather than our protection logic.
  - The half-filled form is protected only after a *trusted* user interaction (rule 8). If the bench fills it through the control channel with JS, rule 8 doesn't fire.
- **Change:**
  1. Define "alive" in §9 as **not discarded and not frozen, with JS running** (checked by a heartbeat the test page reports), and report "frozen" separately.
  2. For each of the 5 tabs, name the §7.3 rule expected to protect it. For the chat tab, either add a rule (e.g. page-reported "live connection with recent incoming messages", same expiry and interaction rules as rule 9) or accept that it is protected only by the list, and use a test site that is *not* on any browser's default list.
  3. Fill the form with OS-level synthesized input in every browser, the same method for all.

---

## Should fix

### Technical accuracy

**S1. §7.2 signal table corrections.** Each cell checked:
- **Linux capture:** the properties are `camera-capture-state`, `microphone-capture-state` and `display-capture-state` (WebKitGTK 2.34+, `WebKitMediaCaptureState`), not `is-capturing-*`. They are native, so mark them ✅ trusted, screen capture included. That makes the page-reported fallback unnecessary on Linux.
- **macOS screen capture:** `cameraCaptureState`/`microphoneCaptureState` are public and KVO-observable (macOS 12+). Display capture is only available as `_displayCaptureState` / `_setDisplayCaptureState:` SPI, so it falls under D7, with the page-reported fallback as the public path.
- **macOS fullscreen:** `WKWebView.fullscreenState` is public and KVO-observable (introduced at WWDC22). Confirm in the SDK header that it's available on macOS 13. Linux: the `enter-fullscreen`/`leave-fullscreen` signals are native (trusted) **(from memory)**.
- **macOS audio:** `requestMediaPlaybackState` (macOS 12+) is a polled request, not an event, and it reports "playing", not "audible". A muted autoplay video counts as playing. Rule 1 ("audible, not user-muted") therefore can't be a trusted signal on macOS without SPI (`_isPlayingAudio` KVO, D7) **(from memory)**. Say so in the table.
- **Windows audio:** the docs say `IsDocumentPlayingAudio` stays true while `IsMuted` is true **(from memory)**. Rule 1 already handles this. Add a note to the cell so the backend doesn't treat "playing" as "audible".
- **Last navigation was POST:** WebView2 `NavigationStarting` args don't expose the HTTP method **(from memory)**. On Windows it comes from the `WebResourceRequested` handler (document-context requests), which we register for the blocker anyway. macOS: `WKNavigationAction.request.httpMethod`. Linux: `webkit_uri_request_get_http_method` on the navigation action's request **(from memory)**. Both are trusted, so drop the "(verify)" there.
- **Windows screen capture:** `ScreenCaptureStarting` exists on `CoreWebView2` and on `CoreWebView2Frame` (fires on `getDisplayMedia`, can be cancelled). It's a start signal only. The end still has to be page-reported, as with mic/camera.

**S2. Hidden views probably can't be captured; capture timing needs a rule (§7.5 "When", §6.3 `snapshot()`).** WebView2 `CapturePreview` captures "what WebView is displaying", and with `IsVisible = false` the WebView "is not rendered". A hidden or suspended view is likely to return a blank or stale image. Capturing "when the user switches away" therefore has to happen *before* the view is hidden, which adds a capture round trip (tens of ms) to every tab switch. Change: capture the **active** tab's screenshot while it's visible, debounced (e.g. 2 s after scroll or input goes quiet, at most every 30 s, and only for pages not excluded by B3). Use the latest one at switch-away, so switching stays instant. WebKitGTK `webkit_web_view_get_snapshot` (cairo surface on 4.1, `GdkTexture` on 6.0) and WKWebView `takeSnapshot(with:)` need the same visible-view assumption until 2a measures them. All three APIs exist and are reachable from Rust (`webview2-com`, `webkit2gtk`, `objc2-web-kit`).

**S3. Capture before discard must have a timeout, and must not let a page veto discarding (§7.5, §7.4).** Capture is an async script round trip through a channel the page can tamper with: on Windows it runs in the main world (S5), and elsewhere wherever 0b/0c find no isolated world. A page that never answers, or answers slowly, must not stall the budget loop or keep itself alive. Change: capture has a fixed timeout (e.g. 500 ms). On timeout, discard anyway and log it. Under budget pressure, prefer candidates with no dirty fields. Also: if the engine resumed a frozen tab by itself (`Resumed{by_engine}`), the freeze-time capture is stale, so capture again before discarding. WebView2's docs say only that "some APIs like Navigate" auto-resume, and recommend checking `IsSuspended`. Whether `ExecuteScript` resumes a suspended view is undocumented, so add it to 0b.

**S4. One environment can host InPrivate *and* spaces, so §6.4 shouldn't create one per profile kind (§6.4, §6.3, §6.8, §8.10).** §6.4 creates "exactly one `CoreWebView2Environment` per profile kind (normal, private)". But `ICoreWebView2ControllerOptions` carries both `ProfileName` and `IsInPrivateModeEnabled`, and Microsoft's multi-profile docs show a normal and an InPrivate controller of the same profile running side by side in one environment. A separate environment means a separate user-data folder, which means a second browser process tree for every private window, the cost v0.3 avoids for spaces. Also, `ProfileKind { Normal, Private }` in the 1a trait will have to become "a space id, optionally private" in 5a. Change:
- §6.4: one environment; private windows and spaces are controller options within it.
- §6.3: make the trait take an opaque `ProfileId` (normal, private, space N) from 1a, with private as a flag, so 5a doesn't refactor every call site. `clear_browsing_data` then takes a `ProfileId` (WebView2 `ICoreWebView2Profile2::ClearBrowsingData` is per profile).

**S5. Isolated worlds and IPC origin checks: Windows and Linux are weaker than §6.6 assumes (§6.6, §7.5).**
- **WebView2:** the reference describes `ExecuteScript` and `AddScriptToExecuteOnDocumentCreated` as running in the document. I found no isolated-world option for host scripts. Assume capture and restore run in the **main world** on Windows, the primary platform. Consequences: the page and its third-party scripts can see and patch the capture script, forge capture payloads (harmless only because restore is same-origin and per-tab) and stall it (S3). An alternative worth a 0b row: ship the page scripts as content scripts of a bundled unpacked extension (`AreBrowserExtensionsEnabled` + `AddBrowserExtension`), because Chromium runs extension content scripts in an isolated world. It's the same mechanism 3a already evaluates for MV3 blocking. The open question is how such a script reaches the host. It must be loaded into every profile and space.
- **WebKitGTK:** `script-message-received` delivers only the message value, and the 2.40 `script-message-with-reply-received` adds only a reply object. Neither identifies the sending frame or origin **(from memory; confirm in 0c)**. That breaks §6.6's "every IPC message is checked against the sender's origin and frame (available on all three backends)". Mitigation: register page-channel handlers **only in our isolated world** (`webkit_user_content_manager_register_script_message_handler_in_world`), so page scripts can't call them, and have our script report its own frame's origin (trustworthy only in the isolated world). Mark the "second layer" as unavailable on WebKitGTK in `EngineCaps`.
- **macOS:** `WKScriptMessage.frameInfo` gives `isMainFrame` and `securityOrigin`, so the check works there.

**S6. Popups with an opener must inherit the opener's view configuration (§6.6 `NewWindowRequested`).** wry 0.55 supports this (`NewWindowResponse::Create { webview }`; Tauri notes the new WebView2 must use the opener's environment). wry 0.53.2 fixed a crash creating popups for webviews with a custom `data_store_identifier`, and a wry issue warns that hand-rolled popups don't inherit the parent's navigation handlers. Change: the popup view gets the opener's `ViewKind::Web` setup (no IPC, blocker filters, navigation policy, permission handler), the **same profile/space and private flag**, and the same page scripts. Test it in 0b with a popup from a private window and, in 5a, from a non-default space.

**S7. Per-space data stores cost more on Linux and macOS than on Windows (§8.10, §6.4, DEC-012).**
- **WebKitGTK 4.1:** `WebKitNetworkSession` (cited by the research report) is a WebKitGTK **6.0** API. On webkit2gtk-4.1, a space is a separate `WebKitWebContext` created with its own `WebKitWebsiteDataManager` (`webkit_web_context_new_with_website_data_manager`, separate base data and cache directories). WebKit developers note that separate contexts need separate network processes (bug 239521), and that two contexts can end up sharing one cookie database by mistake (bug 222366). So each space on Linux adds a network process plus its own web processes. wry's `WebContext::new(Some(dir))` maps to this.
- **macOS 14+:** `WKWebsiteDataStore(forIdentifier:)` exists, and wry exposes it as `data_store_identifier`. Its process cost is unmeasured.
- **Change:** §8.10 "measure the memory cost of each extra WebView2 profile" → measure **per space on all three OSes**, and show it on `lwb://memory`. Name the 4.1 API in §8.10 instead of "(verify … 4.1 vs 6)". Note that wry already exposes `with_profile_name` on Windows (merged May 2026) and `data_store_identifier` on macOS, so 5a needn't hand-roll these. For D12, add a third option: on macOS 13, separate-login spaces can be **ephemeral only** (`nonPersistent()` data store), with persistent spaces from 14.

**S8. Most of the 0b telemetry question is already answered: plan the wording now (§1, §10 0b, §8.6, §5.2 hard rules).** Microsoft's "Data and privacy in WebView2" page says:
- WebView2 collects **required** diagnostic data regardless of the Windows setting;
- optional data follows the Windows *Diagnostics & feedback* setting;
- the embedder controls only SmartScreen and crash reporting.

`IsCustomCrashReportingEnabled` (environment option) defaults to false, meaning "respect OS consent", so WebView2 crash dumps may be uploaded. SmartScreen (`IsReputationCheckingRequired`, **(from memory)** on the API name) sends visited URLs to Microsoft for reputation checks. Changes:
- §8.6 "No automatic upload" must set `IsCustomCrashReportingEnabled = true`, so dumps stay local in `CrashDumpFolderPath`. This is an environment option, so it goes in the single shared environment (§6.4).
- Add **D16: SmartScreen on or off by default**. That's a trade-off between phishing protection and sending URLs to Microsoft.
- Write the §1 / G5 / "what we collect" wording now: "we collect nothing; on Windows, Microsoft's WebView2 runtime sends required diagnostic data that no app can turn off". 0b then confirms it rather than deciding it.

**S9. Reword the "never lose your work" claim so it states its limits (§1, §2 G3, §7.5 "Honest limits").**
- **Absolute wording:** G3 says a hibernated tab "comes back with its form contents and scroll position", and §1 says sleeping tabs "come back where you left them". §7.5 makes this best effort.
- **Missing limits:** the published limits leave out the cases most likely to hit users:
  - a **browser restart or crash** (form data is in memory only, D10; lazy restore starts every tab Discarded; screenshots are purged);
  - a **renderer crash** (nothing was captured);
  - fields excluded on purpose;
  - cross-origin frames;
  - fields skipped by the B2 safety rules.
- **Change:** G3 → "supported form fields and scroll position, best effort". §7.5's limits list gets the items above. Optionally, to cover renderer crashes, capture dirty tabs on input (debounced) and not only at freeze/discard. That costs page-channel traffic, so measure it.

**S10. "100% of supported fields" is self-graded (§4, §10 2a).** The suite is local pages the implementer writes, so 100% is easy on plain HTML and says little about real sites. The "multi-step form" case also has no defined expected outcome, since §7.5 says a server-reset multi-step flow can't be restored. Change:
- Define "comes back": the DOM value matches **and** the page's own model saw it (e.g. a React-controlled input whose submit handler reads state).
- Fix the suite's contents in the spec: plain HTML; React, Vue and Angular controlled inputs; fields rendered after `load`; a same-origin iframe; dynamically added rows; a rich-text editor (via the B2 bar path); and every B2 safety case.
- Define the multi-step expected result (e.g. "step-1 fields restored, step-2 fields offered via the bar").
- Add a separately reported **real-site corpus** score (e.g. 20 popular sites) with no 100% target.
- Keep "100%" out of public copy.

**S11. The memory targets don't fit together, and "most memory-efficient" is hard to prove (§4, §1, DEC-008).**
- **Owner's real-world target:** ≤ 2 GB is 77% of Brave Maximum's 2.6 GB, while the headline realistic-mix target is ≤ 100%. Nothing in the design explains a 23% win in a logged-in session. The opposite is more likely: we keep notification-permission sites (Gmail, Calendar, Slack) undiscarded by rule 7. Restate it as "≤ Brave Maximum in the same session, reporting how many tabs each kept alive", or derive it from 0d.
- **Second line:** "≤ 35% of Brave with Memory Saver off" is looser than what Brave Maximum already achieved on the owner's session (2.6 / 8.1 ≈ 32%). Drop it or tighten it.
- **The superlative:** "the most memory-efficient browser" on macOS invites a comparison with **Safari**, which is the same engine with no extra chrome renderer, and on Windows it rests on parity with Brave Maximum. The headline is the owner's decision. But either the benchmark includes Safari on macOS as a required config, or the public wording is scoped to what the benchmark measures ("as lean as Brave on Maximum, and it never loses your work").
- **Empty window:** ≤ 200 MB on Windows is tight with two renderers (chrome view + new-tab page) plus WebView2's browser, GPU, network and utility processes. 0d should report the breakdown.

### Gaps (daily-driver)

**S12. Web notifications are never delivered (§8.4, §7.3 rule 7, §9).** The spec handles the notification *permission* but never shows notifications.
- **WebView2:** `NotificationReceived` (`ICoreWebView2_24`; when unhandled, WebView2 shows its own UI, which bypasses our "calm" control and per-site settings).
- **WebKitGTK:** the `show-notification` signal **(from memory)**.
- **WKWebView:** has no Web Notifications API. Apple DTS says web push won't work in WKWebView apps, and developers report `window.Notification` is undefined. So on macOS, Gmail/Slack/Calendar notifications need an injected `Notification` shim that forwards to `UNUserNotificationCenter` (page-reported, main world).

Rule 7 and the realistic-mix notification tab mean something different on macOS. Add notification display (OS toast; a click activates the tab) to 1b, and record the macOS limitation.

**S13. Passkeys and security keys likely don't work on macOS builds (§3, §11, §7.3 rule 3).** WebAuthn in a third-party WKWebView for arbitrary sites needs Apple's restricted `com.apple.developer.web-browser.public-key-credential` entitlement. It's granted on request and needs a provisioning profile, which an ad-hoc-signed build doesn't have. Without it, passkey and YubiKey sign-in fails on macOS. Add it to §12 risks and to 0c (test passkey sign-in), and tie it to the signing decision (D8 / Phase 7).

**S14. Single instance and opening URLs from other apps (§8.4 Phase 4 default-browser registration).** A default browser must accept URLs from other apps while it's already running:
- Windows: command line or second instance, handed to the running one;
- macOS: `application:openURLs:` / Apple Events;
- Linux: a `.desktop` `%u` entry plus a single-instance handoff (e.g. D-Bus or a socket).

None of this is in the spec, and "default-browser registration" is useless without it. Add it to 4a, and note that the handoff socket is an untrusted-input parser with a fuzz target, separate from the bench channel.

**S15. DRM on Linux, and the details of the 0b DRM item (§10 0b/0c, §12).** WebKitGTK ships no Widevine CDM, so Netflix, Disney+ and Spotify web don't play on Linux **(from memory)**. Add that to 0c and §12 rather than finding it in 1c. For 0b: little official documentation exists. A Microsoft Teams SDK issue states that WebView2 meets Widevine requirements through the Edge runtime. PlayReady in WebView2 is undocumented. Have 0b also record the **maximum resolution** per service, because Netflix in Edge reaches 1080p+ only with PlayReady **(from memory)**.

### Security and privacy (§7.5 channel)

**S16. Tighten the rules for the capture and restore channel.**
- **Global cap:** in addition to the per-field and per-tab caps, a **global** cap on captured data across all tabs (500 tabs × per-tab cap is a lot of shell RAM), with oldest-first eviction.
- **Origin check at write time:** restore checks the frame's origin in the frame itself at the moment it writes (the restore script compares `location.origin`), and same-origin frames are addressed by engine frame id (WebView2 `ICoreWebView2Frame`). The page might navigate between our check and the write.
- **Screenshot access:** the Internal/chrome view that shows a screenshot gets it via a tab-scoped handle (`lwb://snapshot/<opaque-id>`), never a path. Web views can never reach it (they already block `lwb.localhost`).
- **Moving tabs:** forbid moving a tab between private and normal windows, or between spaces, or drop its captures when it moves (relevant once 4a adds moving tabs between windows).
- **Sensitive fields the exclusions miss:** a password field toggled to `type="text"` by a "show password" button is captured, because `type` is checked at capture time. Exclude any field that was ever `type="password"` or has `autocomplete="current-password"`. Also consider honouring `autocomplete="off"` on fields with `inputmode="numeric"` (common for card verification codes and PINs without `cc-*` tokens), or make it a setting.
- **Sanitizer cost:** "sanitized HTML" for `contenteditable` needs a sanitizer. In Rust that's typically `ammonia` (html5ever, meaningful binary size). Prefer plain text plus the B2 bar, and drop sanitized HTML from v1.

### AI slider and business consistency

**S17. "Rewrite/proofread text in a field" and "translate page" act on the page (§8.8 draft list vs D9, §6.7).** D9 excludes AI that types on the user's behalf. Writing rewritten text into a page field is typing into the page, and translating a page rewrites its DOM. Define the line in §8.8: AI output enters a page only after the user previews it and confirms (that counts as the user's action, through a command), or it goes to the clipboard. Read-only features stay under the slider. Translation either shows the result in our own UI or is listed as a page-modifying feature with its own toggle.

**S18. "No AI code loaded" needs a technical definition, and it affects the size budget (§8.8, D11, §4).** In Rust, code linked statically into the shell *is* in the binary. File-backed pages that never run don't count toward private memory, but "no AI code loaded" would then be false, and the code counts against the ≤ 15 MB installer budget. Change: define "loaded" (a separate DLL/dylib loaded on demand, or a separate helper process; a stable C ABI or IPC either way). D11's "bundle vs download" is then about *shipping*, while "not loaded at No AI" holds in both cases. Include the module size in the §4 installer budget, or exempt it explicitly.

**S19. AI data promises we can't guarantee (§8.8).**
- "Never stored server-side beyond the request" holds only for our proxy. Bring-your-own-key providers, and the upstream model provider behind our credits, have their own retention (often days, for abuse monitoring). Reword it: "our proxy keeps nothing; your provider's retention policy applies; we link it".
- Full AI's proactive features (tab-tidy, a "where was I?" recap, the always-on assistant) send tab titles, URLs or page content without a per-use click. Each feature's toggle should state what leaves the machine.
- "Updates never turn AI on" (§5.2, §8.8) conflicts with "a new AI feature defaults to on for users on Full AI". Word the public promise exactly: "updates never raise your AI level, and new features stay off unless you chose Full AI". Say what a new feature does for users on **Custom** (recommend: off).

### Phase sizing

**S20. 1b, 2a and 4a are each now two or more sessions (§10).** Suggested splits:
- **1b** has storage, fuzzing, settings, session and lazy restore, the 500-tab test, history, downloads, crash page, context menu, permissions, fullscreen, certificate interstitial, HTTP auth, JS dialogs, find, zoom and DevTools. Split it into **1b** (storage, settings, session plus the 500-tab test, history, fuzz targets) and **1c** (the engine-event UIs: dialogs, permissions + notifications (S12), cert/auth, fullscreen, context menu, find, downloads, crash page). The current 1c becomes **1d**.
- **2a** now holds the full state machine, signals on three backends, every protection rule, capture and restore with screenshots on three engines, the encrypted cache (B3), the restore-safety rules (B2) and the suite (S10). Split it into **2a** (lifecycle, signals, protection, `plan()` tests) and **2b** (page-state capture and restore, screenshots, form-restore suite). The current 2b becomes **2c**.
- **4a** holds history and bookmark UIs, autocomplete, private windows, clear data, multiple windows, print and PDF, default browser (+ S14), many-tab UX, auto-archive, GPC, HTTPS-upgrade, onboarding with five importers and their fuzz targets, the command palette, editable shortcuts and naming. Importers alone are substantial: Chromium history DBs are locked while the browser runs (copy first), Firefox `sessionstore.jsonlz4` uses mozlz4, Chromium open tabs live in undocumented SNSS session files, and Arc has different layouts on macOS and Windows. Split it into **4a** (history, bookmarks, autocomplete, private windows, clear data, multiple windows), **4b** (print, PDF, default browser + single instance, many-tab UX, auto-archive, GPC, HTTPS-upgrade, palette, shortcuts) and **4c** (onboarding + importers). Release engineering becomes **4d**.
- **5a** fits 1–2 sessions only if S4's `ProfileId` is in the trait from 1a. Otherwise add a refactor session.

---

## Nice to have

- **N1. Show the restore screenshot in the chrome view, not a new `Internal` view (§7.5, §6.5).** An `Internal` view means a fresh renderer per restore, which costs memory and time against the ≤ 1.5 s restore target. The grown chrome view already exists. Dim the screenshot or show "Loading…" on it, hide it from screen readers (or announce "restoring"), and block clicks and typing on it so input isn't lost into a picture.
- **N2. Job Object/EcoQoS spike details (§10 0b).** WebView2's browser process is launched as a child of the host, so putting *our* process in a job before creating the environment puts the whole tree in it. That gives whole-browser CPU caps and `JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE`, which also fixes the 0d `panic = "abort"` cleanup concern in STATUS.md. Per-renderer caps depend on nested-job rules, because Chromium's sandbox already puts renderers in its own jobs **(from memory)**. Chromium also adjusts process priority itself, so check whether it overrides our EcoQoS settings.
- **N3. Memory-probe fallback (§7.6).** `PROCESS_MEMORY_COUNTERS_EX2` needs Windows 10/11 22H2 **with the September 2023 cumulative update**. Unpatched 22H2 machines, which are likely in the old-PC audience, need a fallback (`PROCESS_MEMORY_COUNTERS_EX` private usage, or `NtQuerySystemInformation` per-process private working set).
- **N4. Define "without hanging" in the 500-tab test (§10 1b).** For example: input-to-paint p95 < 100 ms during restore, the active tab painted in ≤ 2 s, and the memory for 500 discarded tabs (tab-strip DOM + favicons in the chrome view) reported.
- **N5. Mark the "users turn it off" evidence as unverified (§1).** "Users turn the feature off after losing work" rests on the Edge Tech Community quotes, which the report says came from search extracts of a blocked page. DEC-008 kept the quotes out but kept the claim. Footnote it as unverified until Phase 7's spot-check.
- **N6. Arc importer timing (§8.9).** Arc stops working on 22 Dec 2026, before 4a can ship, and uninstalling Arc may remove its local files (especially on Windows). Treat the Arc importer as best effort, cover Arc for Windows' data location too, and cut it first if 4a overruns. The report rated it low priority; DEC-011 records the owner's choice.
- **N7. D13 facts to record.** Apple's Foundation Models framework needs macOS 26 on an Apple Intelligence-capable Mac, and it's a Swift-only API, so reaching it from Rust needs a small Swift shim; objc2 can't bind it directly **(from memory)**. Windows on-device models (e.g. Phi Silica) run only on Copilot+ PCs, and access requirements (package identity / limited-access unlock) must be checked **(from memory)**. On-device OS models run in system processes, so `lwb://memory` should say their memory isn't attributed to us (the transparency principle).
- **N8. Autoplay setting (§8.4).** There's no user autoplay policy setting. WKWebView has `mediaTypesRequiringUserActionForPlayback`, and WebKitGTK has `WebKitWebsitePolicies` autoplay. WebView2 only has the `--autoplay-policy` browser argument, which must be identical across views (§6.4). Add it to 3b or 4.
- **N9. Drag and drop (§8.4 Phase 1).** File drops into pages (Gmail or Drive uploads) must reach the content view. wry's `with_drag_drop_handler` intercepts drops when set. Add it to the 1b/1c checklist alongside tab dragging.
- **N10. Fill the remaining `(verify)` cells (§7.2).** Linux fullscreen and POST, and macOS POST, are known public APIs (S1). Removing those `(verify)` markers leaves 0c's list with only the real unknowns.

---

## Checked and fine
- `WKWebsiteDataStore(forIdentifier:)` is macOS 14+ (WebKit blog), so D12 is correctly framed.
- WebView2 `ProfileName` profiles share one environment and browser process, and InPrivate is a per-controller option (S4).
- `CapturePreview`, `takeSnapshot`, `webkit_web_view_get_snapshot` and `ScreenCaptureStarting` all exist and are reachable from Rust (caveat S2).
- DEC-010's widened D9 and §6.7 are consistent, apart from S17.
- D11 and D13 are consistent with "No AI costs zero RAM", given S18's definition.
- Licensing facts in §5 (FSL → Apache-2.0 after two years, MPL-2.0 file-level copyleft, filter-list licenses) are unchanged and correct.
- The spec correctly kept out the figures the report marks as thin (Windows 10 share, Zen users, Brave Containers).

## Sources
- Chrome performance settings, activities that prevent tab deactivation: https://support.google.com/chrome/answer/12929150
- WebView2 `CapturePreviewAsync`: https://learn.microsoft.com/en-us/dotnet/api/microsoft.web.webview2.core.corewebview2.capturepreviewasync ; `IsVisible` remarks: https://learn.microsoft.com/en-us/dotnet/api/microsoft.web.webview2.core.corewebview2controller.isvisible ; hidden-WebView initialization: https://weblog.west-wind.com/posts/2022/Jul/14/Fighting-WebView2-Visibility-on-Initialization
- WebView2 `TrySuspend` / `IsSuspended`: https://learn.microsoft.com/en-us/dotnet/api/microsoft.web.webview2.core.corewebview2.trysuspendasync , https://learn.microsoft.com/en-us/dotnet/api/microsoft.web.webview2.core.corewebview2.issuspended
- WebView2 data and privacy: https://learn.microsoft.com/microsoft-edge/webview2/concepts/data-privacy ; `IsCustomCrashReportingEnabled`: https://learn.microsoft.com/en-us/microsoft-edge/webview2/reference/win32/icorewebview2environmentoptions3
- WebView2 controller options (`ProfileName`, `IsInPrivateModeEnabled`): https://learn.microsoft.com/en-us/microsoft-edge/webview2/reference/win32/icorewebview2controlleroptions ; multi-profile support: https://learn.microsoft.com/microsoft-edge/webview2/concepts/multi-profile-support
- WebView2 `ExecuteScript` (top-level document): https://learn.microsoft.com/microsoft-edge/webview2/reference/win32/icorewebview2
- WebView2 `NotificationReceived`: https://learn.microsoft.com/en-us/microsoft-edge/webview2/reference/win32/icorewebview2_24 ; `ScreenCaptureStarting`: https://learn.microsoft.com/en-us/dotnet/api/microsoft.web.webview2.core.corewebview2.screencapturestarting
- WebView2 and Widevine (Microsoft Live Share SDK issue): https://github.com/microsoft/live-share-sdk/issues/14
- `PROCESS_MEMORY_COUNTERS_EX2` requirements: https://learn.microsoft.com/en-us/windows/win32/api/psapi/ns-psapi-process_memory_counters_ex2
- wry `NewWindowResponse`: https://docs.rs/wry/0.55.0/wry/enum.NewWindowResponse.html ; wry 0.53.2 notes: https://v2.tauri.app/release/wry/v0.53.2 ; wry `with_profile_name` PR: https://github.com/tauri-apps/wry/pull/1738
- WebKitGTK capture-state properties: https://webkitgtk.org/reference/webkit2gtk/stable/property.WebView.camera-capture-state.html , https://webkitgtk.org/reference/webkit2gtk/stable/property.WebView.display-capture-state.html
- WebKitGTK script message reply (2.40): https://webkitgtk.org/reference/webkit2gtk/stable/signal.UserContentManager.script-message-with-reply-received.html
- WebKitGTK contexts and network processes: https://bugs.webkit.org/show_bug.cgi?id=239521 , https://bugs.webkit.org/show_bug.cgi?id=222366 ; `WebKitWebContext`: https://webkitgtk.org/reference/webkit2gtk/2.42.4/class.WebContext.html
- WebKit profiles API (macOS 14): https://webkit.org/blog/14423/building-profiles-with-new-webkit-api/
- WKWebView capture state API: https://bugs.webkit.org/show_bug.cgi?id=222166 ; display-capture SPI in a 2024 changeset: https://lists.webkit.org/pipermail/webkit-changes/2024-September/307345.html ; `fullscreenState` (WWDC22): https://developer.apple.com/videos/play/wwdc2022/10049/
- WKWebView and web notifications: https://developer.apple.com/forums/thread/760767
- WKWebView WebAuthn entitlement: https://developer.apple.com/forums/thread/829169 , https://developer.apple.com/forums/thread/774904 , https://passkeys.dev/docs/reference/macos/
