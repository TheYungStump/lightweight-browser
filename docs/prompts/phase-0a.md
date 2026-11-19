# Prompt: Phase 0a, repo foundation

Paste the following into a fresh local Claude Code session in this repo (on `main`, pulled).

---

You are implementing **Phase 0a (repo foundation)** of a low-memory desktop browser written in Rust. Read `CLAUDE.md`, then `docs/SPEC.md` (at least §5.1, §6.1, §6.2 and §10 Phase 0a), `docs/STATUS.md` and `docs/DECISIONS.md`.

Work on a new branch `phase-0a-foundation` from `main`.

Deliver:
1. **Cargo workspace** matching SPEC §6.2: crates `lwb-app` (binary that prints a version line and exits for now), `lwb-core`, `lwb-engine`, `lwb-memprobe`, `lwb-blocker`, `lwb-storage` (libraries with a placeholder module and one trivial test each), plus `tools/lwb-bench` (binary stub). Exclude `spikes/*` from the workspace (spikes are standalone projects). Edition 2024; set `rust-version` to the current stable. Shared lints in `[workspace.lints]` (deny `unsafe_op_in_unsafe_fn`; clippy `all` warn). Add a release profile tuned for size and memory (`lto = "thin"`, `codegen-units = 1`, `panic = "abort"` is fine for the app binary, `strip = true`), and say in the PR why each setting was chosen.
2. **Tooling:** `rustfmt.toml`, `.editorconfig`, `rust-toolchain.toml` pinned to stable, `deny.toml` for `cargo-deny`: license allow-list (MIT, Apache-2.0, BSD-2/3, ISC, Zlib, Unicode-3.0, MPL-2.0, LGPL only for system libraries; deny GPL/AGPL in our dependency tree) + advisories + bans on duplicate major versions as warnings.
3. **License and contribution files:** `LICENSE` with the official **FSL-1.1-ALv2** text, fetched from the official source (fsl.software / the getsentry/fsl.software repo). Fill in the licensor as the GitHub account owner and the year 2026; ask the owner if the licensor name should differ. Add `CONTRIBUTING.md` (how to build, PR rules from CLAUDE.md, CLA requirement), a CLA document (`docs/CLA.md`, a standard individual CLA such as the Apache ICLA adapted to this project, for the owner to approve), and a short README section on licensing. Explain in the PR how the owner enables the CLA Assistant GitHub app; don't try to do it yourself.
4. **Issue templates** in `.github/ISSUE_TEMPLATE/`: bug report, feature request, tester feedback (OS, version, memory observations).
5. **CI** in `.github/workflows/ci.yml`: matrix `windows-latest`, `macos-latest`, `ubuntu-latest`; steps: checkout, Rust toolchain, cache (Swatinem/rust-cache), `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo build --workspace`, `cargo test --workspace`; a separate `cargo-deny` job on Linux. The Linux job installs `libwebkit2gtk-4.1-dev libgtk-3-dev libsoup-3.0-dev` (and anything else wry's current docs require), even though nothing links them yet, so the 1c change is trivial. Pin third-party actions to a major version or commit SHA. Trigger on PRs and pushes to `main`; cancel superseded runs.
6. **CLAUDE.md:** fill in the "Build and test commands" section.
7. **STATUS.md:** mark 0a done with a short summary.

Constraints: no product code beyond stubs. No new dependencies beyond what the stubs need (ideally none). Run fmt, clippy and test locally before committing.

Acceptance: CI green on all three OSes on the PR; `cargo deny check` passes. Open a PR to `main` titled "Phase 0a: repo foundation" listing what was done, anything the owner must do by hand (CLA Assistant, branch protection), and any deviations recorded in DECISIONS.md.
