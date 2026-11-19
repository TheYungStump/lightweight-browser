# Contributing

Thanks for helping. This is a low-memory desktop browser written in Rust. **Memory is the product**, so every change is judged partly on what it costs in RAM and binary size.

## Before you start
- Read [`docs/SPEC.md`](docs/SPEC.md). It's the source of truth. [`docs/STATUS.md`](docs/STATUS.md) says what's in progress, and [`docs/DECISIONS.md`](docs/DECISIONS.md) lists agreed deviations from the spec.
- For anything bigger than a small fix, open an issue first, so we can agree on the approach before you write code.

## Building
You need [rustup](https://rustup.rs). The toolchain is pinned in `rust-toolchain.toml`, and rustup installs it automatically the first time you build.

On Linux, also install the WebKitGTK development packages (Ubuntu 22.04/24.04):

```bash
sudo apt-get install -y libwebkit2gtk-4.1-dev libgtk-3-dev libsoup-3.0-dev build-essential pkg-config
```

On Windows you need the MSVC build tools (Visual Studio Build Tools with "Desktop development with C++", including a Windows 11 SDK). Some dependencies compile C code. If a build fails with `C1083: Cannot open include file` for a header such as `corecrt_wstdio.h`, the SDK install is incomplete: repair it from Settings → Apps → "Windows Software Development Kit". The WebView2 runtime ships with Windows 10/11.

```bash
cargo build --workspace
```

```bash
cargo run -p lwb-app
```

## Before every commit
CI runs the same checks on Windows, macOS and Linux:

```bash
cargo fmt --all
```

```bash
cargo clippy --workspace --all-targets -- -D warnings
```

```bash
cargo test --workspace
```

License and advisory checks (install once with `cargo install cargo-deny --locked`):

```bash
cargo deny check
```

## Pull request rules
- Work on a branch and open a PR to `main`. Never push to `main` directly. Phase work uses the branch `phase-<id>-<slug>`.
- **Don't silently diverge from the spec.** If an API doesn't exist or a design doesn't work, record the problem and the alternative you chose in `docs/DECISIONS.md`, and say so in the PR description.
- `lwb-core` stays free of OS and engine dependencies. Policy logic is pure and unit-tested with an injected clock.
- Platform-specific code lives only in `crates/lwb-engine/src/{windows,macos,linux}/` and `crates/lwb-memprobe`. Every `unsafe` block needs a `// SAFETY:` comment.
- Content webviews never get privileged IPC. Messages from injected page scripts are untrusted input (SPEC §6.6).
- Avoid heavy dependencies (no full tokio runtime, no big UI frameworks) unless the spec or a recorded decision allows them. Call out any new dependency that adds meaningful binary size or RAM in the PR.
- Never commit filter lists, API keys, secrets or large binaries.
- Keep `docs/STATUS.md` up to date when you finish or hand over a piece of phase work.

## Contributor License Agreement
Before your first PR can be merged you need to sign the [Contributor License Agreement](docs/CLA.md). The CLA Assistant bot comments on your PR with a link, and signing takes one click. You sign once, and it covers all your future contributions.

Why: the project is licensed under the Functional Source License (see [README](README.md#license)), and each release becomes Apache-2.0 after two years. The CLA lets the owner keep relicensing the code like that, and dual-license it if needed, without tracking down every contributor. You keep the copyright to your work.

## Release profile
The release profile in `Cargo.toml` is tuned for small binaries and low memory:

| Setting | Why |
|---|---|
| `opt-level = 3` | Full speed optimisation. The shell is small, so size-focused levels (`s`/`z`) save little and cost speed on hot paths like ad-block lookups. We'll revisit this with real numbers from the size budget (SPEC §4). |
| `lto = "thin"` | Cross-crate inlining and dead-code removal for most of the size and speed gain of full LTO, with much faster link times. |
| `codegen-units = 1` | Lets the optimiser see each crate as a whole: smaller, faster code in exchange for slower release builds. |
| `panic = "abort"` | Drops the unwinding tables and landing pads (smaller binary, less memory). A panic in the shell is fatal anyway, and SPEC §8.6 logs it from a panic hook before the process exits. Tests still unwind, because Cargo ignores this setting for the test profile. |
| `strip = true` | Removes symbols and debug info from the shipped binary, often the largest single size saving. Crash diagnostics come from our panic log rather than symbols in the binary. |
