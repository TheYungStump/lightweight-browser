# Instructions for Claude Code sessions

This repo is a lightweight, low-memory desktop browser in Rust (Windows primary, macOS and Linux too).

## Before you write any code
1. Read `docs/SPEC.md`. It is the source of truth. Read in full at least §6 (architecture), §7 (memory management), and the section for your phase in §10.
2. Read `docs/STATUS.md` (current phase, handoff notes) and `docs/DECISIONS.md` (deviations already agreed).
3. If your session was started from a prompt in `docs/prompts/`, follow it.

## Rules
- **Don't silently diverge from the spec.** If an API doesn't exist or a design doesn't work, record the problem and your chosen alternative in `docs/DECISIONS.md` and call it out in the PR description.
- Work on the phase branch named in the prompt (`phase-N-<slug>`). Never push to `main` directly.
- Keep `docs/STATUS.md` up to date: what's done, what's in progress, what's next. If your context is getting long, stop at a clean point and write a handoff note there.
- `lwb-core` must stay free of OS and engine dependencies. Policy logic is pure and unit-tested with an injected clock.
- Platform-specific code lives only in `crates/lwb-engine/src/{windows,macos,linux}/` and `crates/lwb-memprobe`. Every `unsafe` block needs a `// SAFETY:` comment.
- Content webviews never get privileged IPC. Messages from injected scripts are untrusted input (SPEC §6.6).
- Memory is the product. Avoid heavy dependencies (no full tokio runtime, no big UI frameworks) unless the spec or a recorded decision allows it. Mention in the PR any new dependency that adds meaningful binary size or RAM.
- Before committing: `cargo fmt --all`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`.
- Never commit filter lists, API keys, or other secrets/large binaries.
- Don't include AI model names or identifiers in commits, code comments, or docs.
