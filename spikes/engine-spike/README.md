# engine-spike (Phase 0b, Windows)

Throwaway spike that checks the WebView2 APIs SPEC §10 Phase 0b depends on. Results: `docs/spikes/phase0b-windows.md`. Manual steps: `docs/testing/phase-0b.md`. Not part of the workspace, so build it from this folder.

```powershell
cargo run --release                                     # interactive
cargo run -- --script scripts\events.txt --tag events   # scripted run
```

Everything it writes (WebView2 profile, logs, downloaded filter lists, the generated test extension, screenshots) goes to `%LOCALAPPDATA%\lwb-spike`, never into the repo.

## Layout
- `src/main.rs`: window, chrome view, content views, popups, and the text-command dispatcher (`App::cmd`). Flags are documented at the top.
- `src/wv2.rs`: direct WebView2 access via `webview2-com`: environment creation, every engine event handler, suspend/resume, memory target, process info, extensions.
- `src/winproc.rs`: private working set / commit, CPU time, Job Object, EcoQoS.
- `src/blocker.rs`: adblock-rust engine and latency stats. `src/osinput.rs`: real OS input for tests. `src/server.rs`: local test server for `pages/`.
- `ui/chrome.html`: the chrome strip and fake autocomplete dropdown (overlay test).
- `ext/`: the MV3 test extension (isolated-world check, DEC-015). `token.js` is generated per run.
- `scripts/`: command scripts (`wait <ms>` pauses; `@/path` = local test server). `tools/`: network observation for the telemetry check.

## Commands (chrome buttons and scripts)
`tab N`, `nav URL`, `navv N URL`, `new [private] URL`, `close N`, `suspend N`, `resume N`, `state`, `exec N JS`, `docscript N JS`, `memlevel N low|normal`, `mute N on|off`, `click|rclick N SEL` (DevTools input), `osclick N SEL`, `osclickxy X Y`, `oskey L` (OS input), `procs [label]`, `procx`, `overlay on|off [noveil]`, `blocker load|on|off|reset|drop` / `blocker` (stats), `capture allow|cancel`, `ext`, `job`, `cpucap PCT`, `cpu start|end`, `eco on|off|get`, `crashinfo`, `bounds`, `video`, `shot NAME`, `log TEXT`, `quit`.
