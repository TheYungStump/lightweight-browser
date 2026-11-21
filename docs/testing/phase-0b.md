# Manual test checklist: Phase 0b (Windows engine spike)

Run on Windows, on the `phase-0b-windows-spike` branch. Section A is the hands-on part of the spike itself: these checklist items need your eyes, hands or accounts. Record what you see next to each step (or just tell the session) and it goes into `docs/spikes/phase0b-windows.md`. Section B re-runs the automated scripts.

**Start the spike** (first build takes a couple of minutes):

```powershell
cd spikes\engine-spike
cargo run
```

(A debug build is fine for these checks: the web engine is the installed WebView2 runtime, not our code.)

A window opens with a grey strip at the top (the "chrome view") and a page below it. Buttons in the strip: **Tab 0 … Tab 4** switch tabs, the address bar navigates (type a URL, press Enter), **State / Memory / Bounds / Video info** write results to the console window and to `%LOCALAPPDATA%\lwb-spike\logs\spike-main.log`. Popups open as new tabs.

## A. Hands-on items

### A1. Google sign-in popup (SPEC §6.6, §10 0b)
1. In the address bar type `www.reddit.com/login` and press Enter. Click **Continue with Google**.
   - If Reddit doesn't show a popup (it sometimes uses a redirect), use `www.canva.com/login` → **Continue with Google** instead.
2. [ ] A new tab opens with Google's sign-in page (that's the popup). Note which tab number it became.
3. [ ] Sign in with your Google account (password, 2FA as usual).
4. [ ] The popup tab closes by itself and you're back on the opener tab.
5. [ ] The opener page shows you as signed in (it may take a second to reload).
6. Repeat from a **private** tab: click **New private tab**, then do steps 1–5 in that tab.
   - [ ] Popup opens, sign-in completes, popup closes, opener is signed in.
   - [ ] Close the spike, start it again, open the same site in a normal tab: you are **not** signed in from the private session (private data wasn't kept). (You may still be signed in from step 5 of the normal-tab test; that's expected.)
7. Afterwards, sign out of Reddit/Canva in the spike if you like. The spike's profile lives in `%LOCALAPPDATA%\lwb-spike\profile-main`; delete that folder to wipe it.

### A2. DRM video (Netflix, Disney+, Spotify)
For each service: navigate to it in the address bar, sign in, start playing something, let it play ~30 s at full window size, then click **Video info**.
- [ ] **Netflix** (`www.netflix.com`): plays? Press **Ctrl+Alt+Shift+D** on the video to show Netflix's overlay and note the **playing resolution** (e.g. `1920x1080`) and anything that says "PlayReady" or "Widevine". Also note what **Video info** logged.
- [ ] **Disney+** (`www.disneyplus.com`): plays? Note **Video info** (`w`/`h` = the resolution being decoded).
- [ ] **Spotify** (`open.spotify.com`): a song plays? (Audio only; Spotify web needs DRM even for audio.)
- [ ] In the console, copy the lines that start with `WebMessage ... EME granted` / `EME denied`: they show which DRM system each service asked for and got.
- If a service shows an error code instead of playing, note the code.

### A3. HiDPI: 100%, 150% and dragging between monitors
Set your second monitor to 150%:
1. Right-click the desktop → **Display settings**.
2. At the top, click the box labelled **2** (the second monitor). If unsure which is which, click **Identify**.
3. Under **Scale & layout**, set **Scale** to **150%**. Leave monitor 1 at 100%.

Then, with the spike running:
- [ ] On monitor 1 (100%): the strip and page line up (no gap or overlap between the grey strip and the page), text is crisp. Click **Bounds**.
- [ ] Drag the window to monitor 2 (150%). While and after dragging: does the page stay lined up under the strip? Any moment where it's the wrong size or offset? Text crisp after the move? Click **Bounds**.
- [ ] Drag it back to monitor 1. Same checks. Click **Bounds**.
- [ ] Maximise on each monitor, check again.
- Set monitor 2 back to 100% afterwards (same steps).

### A4. Overlay flicker (SPEC §6.5)
1. Navigate to a busy page, e.g. `www.youtube.com`, and start a video playing.
2. Click in the address bar and type a few letters: a fake autocomplete dropdown appears over the page. Press **Escape** to close it. Do this ~10 times.
3. Also click **Overlay test** repeatedly (it toggles the dropdown).
- [ ] Any flicker, white flash, or black flash on the page when the dropdown opens or closes?
- [ ] Does the video keep playing smoothly behind the dropdown?
- [ ] Is the page visible through the area around the dropdown (it should be: only the dropdown itself is drawn)?
- [ ] Clicking on the page area while the dropdown is open closes the dropdown (the click doesn't reach the page; that's expected).

### A5. Elevated network capture for the telemetry check (optional but recommended)
This adds DNS query names and a packet capture to the non-admin observation already done, and tests the one documented control that needs admin rights: the WebView2 policy `ExperimentationAndConfigurationServiceControl = 0`. The script sets that policy for one extra run and then restores it. It takes ~11 minutes and the spike windows open minimised.
1. Close other browsers, Teams/Slack/Discord, and anything else that's online, so the capture is quiet.
2. Open **PowerShell as Administrator** (Start → type PowerShell → right-click → Run as administrator).
3. Run (adjust the path if your repo is elsewhere):

```powershell
cd "C:\Users\Brandon\Desktop\AI Businesses\lightweight-browser\spikes\engine-spike"
```

```powershell
powershell -ExecutionPolicy Bypass -File .\tools\observe-elevated.ps1
```

4. [ ] It ends with `Done. Results in ...`. Tell the session the folder path. Don't commit anything from it.

## B. Automated scripts (re-run to reproduce the report's numbers)
From `spikes\engine-spike` (each writes `%LOCALAPPDATA%\lwb-spike\logs\spike-<tag>.log`):
- [ ] `cargo run -- --script scripts\lifecycle.txt --tag lifecycle`: TrySuspend succeeds on the hidden view and the heartbeat pings stop; ExecuteScript on the suspended view returns and the view stays suspended; TrySuspend on the visible view fails with `0x8007139F`.
- [ ] `cargo run -- --script scripts\events.txt --tag events`: POST is logged as `POST document`; worker requests show `srcKind=2` and `srcKind=4`; basic auth answered; certificate error cancelled; internal host blocked; permissions, screen capture, context menu and both popups (normal and private) report `opener=true`.
- [ ] `cargo run -- --script scripts\osinput.txt --tag osinput`: fullscreen enters and exits; Ctrl+T/L/W are handled while a page input has focus. (This one moves your mouse and types; don't touch the keyboard while it runs.)
- [ ] `cargo run -- --ext --script scripts\isolated2.txt --tag isolated2`: `CHANNEL message token_ok=true` for the extension, `token_ok=false` for the page's forgery.
- [ ] `cargo run --release -- --script scripts\adblock.txt --tag adblock`: downloads EasyList + EasyPrivacy into `%LOCALAPPDATA%\lwb-spike\lists` and prints handler p50/p99 per site.
- [ ] `cargo run --release -- --job --script scripts\job.txt --tag job`: every WebView2 process is "in our job"; the 1% cap cuts the renderer's CPU time sharply.
- [ ] `git status` shows no filter lists, profiles, logs or screenshots in the repo (they all live under `%LOCALAPPDATA%\lwb-spike`).
