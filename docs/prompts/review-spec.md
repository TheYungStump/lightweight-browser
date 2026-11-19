# Prompt: spec review session

Paste the following into a fresh Claude Code session in this repo.

---

You are reviewing the project specification for a low-memory desktop browser before any code is written. Read `docs/SPEC.md`, `CLAUDE.md`, `docs/STATUS.md` and `docs/DECISIONS.md`.

Review the spec critically, as a senior browser/systems engineer would. Don't write code. Focus on:

1. **Technical accuracy.** Check every claimed platform API against current documentation (WebView2, WKWebView, WebKitGTK, wry, tao, adblock-rust): it must exist, behave as described, and be reachable from Rust. Especially check the §7.2 signal table, §6.6 (popups with `window.opener`, isolated worlds), §7.6 (memory attribution, especially macOS XPC processes) and §8.1 (blocking approach per OS). Cite sources.
2. **Feasibility of the memory targets** in §4, given that WebView2 is Chromium-based. Are any unrealistic? Are any too easy?
3. **Gaps.** Anything a daily-driver browser needs that's missing from Phases 1–4 (e.g. permissions prompts, certificate errors, print, HiDPI/multi-monitor, accessibility, IME input, drag & drop, PDF viewing, clipboard, autoplay policy, default-browser registration).
4. **Hibernation safety** (§7.3): cases where a tab would wrongly be hibernated.
5. **Security model** (§6.6): weaknesses in how content scripts, `lwb://` pages and IPC are separated.
6. **Phase sizing:** is each phase a sensible size for 1–3 implementation sessions? Suggest splits.
7. **Licensing/business** (§5): factual errors only (not legal advice).

Also read earlier reviews in `docs/reviews/` and don't repeat findings already addressed.

Output a review in `docs/reviews/spec-review-<next number>.md` with: a summary verdict (approve / approve with changes / rework), then findings grouped as **Blocking**, **Should fix**, **Nice to have**. Each finding gets the spec section, the problem, evidence/source, and a concrete suggested change. Commit it on a branch named `review/spec-<next number>` and push. Don't edit `SPEC.md` yourself.
