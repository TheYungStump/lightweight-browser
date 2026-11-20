# Manual test checklist: Phase 0a (repo foundation)

Run on Windows from a fresh clone of the PR branch. Every step should take under a minute after the first build.

## Toolchain and build
- [ ] `rustup show` in the repo root reports toolchain `1.100.0` (from `rust-toolchain.toml`). On a machine without it, the first `cargo` command installs it automatically.
- [ ] `cargo build --workspace` succeeds with no warnings.
- [ ] `cargo run -p lwb-app` prints `lwb-app 0.0.0` and exits.
- [ ] `cargo run -p lwb-bench` prints `lwb-bench 0.0.0` and exits.
- [ ] `cargo build --release -p lwb-app` succeeds. Note the size of `target/release/lwb-app.exe` in the PR (a baseline for the SPEC §4 size budget).

## Checks
- [ ] `cargo fmt --all --check` reports nothing.
- [ ] `cargo clippy --workspace --all-targets -- -D warnings` passes.
- [ ] `cargo test --workspace` runs 7 tests, all passing (one per crate).
- [ ] `cargo deny check` passes (install once with `cargo install cargo-deny --locked`).

## Lints actually apply
- [ ] Temporarily add `pub unsafe fn f(p: *const u8) -> u8 { *p }` to `crates/lwb-core/src/lib.rs`, then run `cargo build -p lwb-core`. It **fails** with `unsafe_op_in_unsafe_fn`. Revert.

## Repository files
- [ ] `LICENSE` matches the official FSL-1.1-ALv2 template (getsentry/fsl.software, `FSL-1.1-ALv2.template.md`), with `Copyright 2026 TheYungStump` as the only change.
- [ ] `README.md`, `CONTRIBUTING.md` and `docs/CLA.md` render properly on GitHub, and their links work.
- [ ] On GitHub, "New issue" offers exactly three templates: Bug report, Feature request, Tester feedback. Blank issues are disabled.

## CI
- [ ] On the PR, the `windows-latest`, `macos-latest`, `ubuntu-latest` and `cargo-deny` jobs are all green.
- [ ] Pushing a second commit to the PR cancels the run still in progress for the first.
