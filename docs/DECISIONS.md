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
