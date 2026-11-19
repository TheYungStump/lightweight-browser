# Prompt: Phase 0b, Windows engine spike

Paste the following into a fresh local Claude Code session **on the owner's Windows machine**, in this repo (on `main`, pulled). It can run in parallel with Phase 0a.

---

You are implementing **Phase 0b (Windows engine spike)** of a low-memory desktop browser written in Rust. This is the go/no-go check for our primary platform: we're proving that the engine APIs the spec depends on exist and behave as described, **before** we build on them. Read `CLAUDE.md`, then `docs/SPEC.md` in full (especially §6.3–6.6, §7.1–7.2, §7.6, §8.1 and §10 Phase 0b), `docs/STATUS.md` and `docs/DECISIONS.md`.

Work on a new branch `phase-0b-windows-spike` from `main`.

**First follow `docs/prompts/_kickoff.md`: do the orientation check, post your plan and questions, and wait for the owner's go-ahead before writing any code.**

Build a **standalone** throwaway app in `spikes/engine-spike/` (its own `Cargo.toml`, not part of the workspace) using `tao` + `wry` (current versions), plus `webview2-com`/`windows` via wry's `WebViewExtWindows` for anything wry doesn't expose. Code quality can be spike-grade, but every `unsafe` needs a `// SAFETY:` comment, and the code must be readable enough for Phase 1a to borrow from.

The app: one window with a chrome view (simple HTML strip: two "tab" buttons, an address input, a few test buttons) and 2+ content views below it, switchable. Add whatever test buttons/logging you need. Log results to the console and to a log file.

For **every item in the SPEC §10 Phase 0b checklist**, determine ✅ works / ⚠️ works with caveats / ❌ doesn't work, and write `docs/spikes/phase0b-windows.md` with a table: item, result, how you tested it, notes/caveats, code pointer (file:line in the spike), and the docs link for the API used. Extra guidance:
- **Measure, don't guess.** For memory items, report actual numbers (private working set per process, summed over our process + the WebView2 processes from `GetProcessExtendedInfos`), with the exact method. Note the WebView2 runtime version and Windows build.
- **Google sign-in popup:** test with a real "Sign in with Google" flow (e.g. a site using Google Identity Services). Confirm the popup opens, completes and returns to the opener page. The owner may need to do the sign-in step by hand. If so, write clear instructions and ask them.
- **Ad-block items:** add `adblock` (adblock-rust) to the spike only. Load EasyList + EasyPrivacy **downloaded at runtime** into a cache dir outside the repo (never commit lists). Measure the engine's resident memory and the per-request handler latency (p50/p99 over a heavy page like a news site).
- **Overlay test:** grow the chrome view over the content area with a transparent background, show a fake autocomplete dropdown on top of a loaded page, shrink back. Report flicker, transparency and click-through behaviour.
- **HiDPI:** test at 100% and 150% scaling and while dragging between monitors if the owner has two. Otherwise say so.
- For anything that needs the owner's hands (sign-in, second monitor, watching for flicker), list the steps and ask the owner rather than skipping.

Finish with:
1. A **summary** at the top of the report: go / no-go for Windows, and the 3–5 most important findings.
2. A `docs/DECISIONS.md` entry for every ❌ and every meaningful ⚠️: the problem, your recommended workaround (direct bindings, alternative API, scope change) and the SPEC sections affected. **Don't edit SPEC.md**; the orchestrator does that.
3. STATUS.md updated (0b done, link to the report).
4. A PR to `main` titled "Phase 0b: Windows engine spike" summarising the go/no-go and the decisions needing owner/orchestrator input.

Don't start any Phase 1 work. If a checklist item turns out to be much harder than expected, time-box it, record what you learned, and move on.
