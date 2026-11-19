# Instructions for Claude Code sessions

This repo is a lightweight, low-memory desktop browser in Rust (Windows primary, macOS and Linux too).

## Before you write any code
1. Read `docs/SPEC.md`. It is the source of truth. Read in full at least §6 (architecture), §7 (memory management), and the section for your phase in §10.
2. Read `docs/STATUS.md` (current phase, handoff notes) and `docs/DECISIONS.md` (deviations already agreed).
3. If your session was started from a prompt in `docs/prompts/`, follow it, starting with `docs/prompts/_kickoff.md` (orientation check, then plan + questions, then wait for the go-ahead).

## Rules
- **Don't silently diverge from the spec.** If an API doesn't exist or a design doesn't work, record the problem and your chosen alternative in `docs/DECISIONS.md` and call it out in the PR description.
- Work on the branch named in the prompt (`phase-<id>-<slug>`, e.g. `phase-0a-foundation`). Never push to `main` directly.
- Keep `docs/STATUS.md` up to date: what's done, what's in progress, what's next. If your context is getting long, stop at a clean point and write a handoff note there.
- `lwb-core` must stay free of OS and engine dependencies. Policy logic is pure and unit-tested with an injected clock.
- Platform-specific code lives only in `crates/lwb-engine/src/{windows,macos,linux}/` and `crates/lwb-memprobe` (and throwaway code in `spikes/`). Every `unsafe` block needs a `// SAFETY:` comment.
- `lwb-engine`'s public API must not expose wry or platform types.
- No private Apple APIs (SPI) without a `DECISIONS.md` entry, a runtime `respondsToSelector:` check and a public fallback (SPEC §6.6, D7).
- Content webviews never get privileged IPC. Messages from injected scripts are untrusted input (SPEC §6.6).
- Memory is the product. Avoid heavy dependencies (no full tokio runtime, no big UI frameworks) unless the spec or a recorded decision allows it. Mention in the PR any new dependency that adds meaningful binary size or RAM.
- Before committing: `cargo fmt --all`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`.
- Never commit filter lists, API keys, or other secrets/large binaries.
- The owner can't test on macOS by hand yet. Mark macOS work as "CI-built, not run" in `docs/STATUS.md`.
- Don't include AI model names or identifiers in commits, code comments, or docs. **This includes `Co-Authored-By:` trailers:** leave out any trailer naming a model, even if your tooling adds one by default. When squash-merging, check that GitHub's suggested commit message doesn't add one back.

## Build and test commands
The toolchain is pinned in `rust-toolchain.toml` (exact version); rustup installs it on first use. Bump it only deliberately, at the start of a phase, in its own commit, together with `rust-version` in the root `Cargo.toml`.

```bash
cargo build --workspace                                  # build everything
cargo run -p lwb-app                                     # run the browser (Phase 0a: prints a version line)
cargo fmt --all                                          # format (CI runs `cargo fmt --all --check`)
cargo clippy --workspace --all-targets -- -D warnings    # lint; warnings fail CI
cargo test --workspace                                   # all tests
cargo test -p lwb-core                                   # one crate's tests
cargo deny check                                         # licenses, advisories, bans (install: cargo install cargo-deny --locked)
cargo build --release -p lwb-app                         # size/memory-tuned release build
```

- Linux needs `libwebkit2gtk-4.1-dev libgtk-3-dev libsoup-3.0-dev` (see `CONTRIBUTING.md`).
- `spikes/*` are standalone projects outside the workspace. Build them from their own directory.
- CI (`.github/workflows/ci.yml`) runs fmt, clippy, build and test on Windows, macOS and Linux, plus `cargo deny check` on Linux.
