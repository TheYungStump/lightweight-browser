//! Phase 0b Windows engine spike (throwaway). One window: an HTML chrome strip
//! on top, switchable content views below, all on ONE shared WebView2
//! environment. Every test is a text command (from chrome buttons, or from a
//! `--script` file), and results go to the console and the log file.
//!
//! Flags:
//!   --script <file>   run commands from a file (`wait <ms>` lines pause)
//!   --no-chrome       no chrome view (memory baseline)
//!   --job             put our process in a Job Object before creating WebView2
//!   --no-crash-local  leave IsCustomCrashReportingEnabled off
//!   --ext             enable browser extensions in the environment
//!   --args "<flags>"  WebView2 AdditionalBrowserArguments (default: autoplay flag)
//!   --url <url>       first content view URL (default: local test index)
//!   --views <n>       number of initial content views (default 2)
//!   --tag <name>      log file name suffix and profile folder suffix
//!   --no-reputation   IsReputationCheckingRequired = false (SmartScreen off via the API)

#![cfg(windows)]

mod blocker;
mod logging;
mod osinput;
mod server;
mod winproc;
mod wv2;

use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::time::Duration;

use tao::dpi::{LogicalPosition, LogicalSize};
use tao::event::{Event, WindowEvent};
use tao::event_loop::{ControlFlow, EventLoopBuilder, EventLoopProxy};
use tao::window::{Window, WindowBuilder};
use tracing::{info, warn};
use webview2_com::Microsoft::Web::WebView2::Win32::{ICoreWebView2, ICoreWebView2Environment};
use windows::Win32::UI::WindowsAndMessaging::{
    HWND_TOP, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE, SetWindowPos,
};
use wry::{
    NewWindowResponse, Rect, WebView, WebViewBuilder, WebViewBuilderExtWindows, WebViewExtWindows,
};

const CHROME_H: f64 = 84.0;
const CHROME_HTML: &str = include_str!("../ui/chrome.html");

#[derive(Debug)]
pub enum UserEvent {
    Cmd(String),
}

/// State shared with WebView2 callbacks (UI thread only, hence Rc/RefCell).
pub struct Shared {
    pub proxy: EventLoopProxy<UserEvent>,
    pub env: ICoreWebView2Environment,
    pub blocker: RefCell<Option<blocker::Blocker>>,
    pub blocker_on: Cell<bool>,
    pub verbose_requests: Cell<bool>,
    pub allow_capture: Cell<bool>,
    pub window: Rc<Window>,
    pub port: u16,
    /// Popup views created inside NewWindowRequested, adopted as tabs by the loop.
    pub pending_popups: RefCell<Vec<View>>,
    pub next_id: Cell<usize>,
    pub ext_token: RefCell<String>,
    pub no_reputation: bool,
}

pub struct View {
    pub id: usize,
    pub wv: WebView,
    pub core: ICoreWebView2,
    pub private: bool,
    /// For popups: the id of the view that opened it.
    pub opener: Option<usize>,
}

struct Opts {
    script: Option<String>,
    no_chrome: bool,
    job: bool,
    crash_local: bool,
    ext: bool,
    args: String,
    url: Option<String>,
    views: usize,
    tag: String,
    no_reputation: bool,
}

fn parse_opts() -> Opts {
    let mut o = Opts {
        script: None,
        no_chrome: false,
        job: false,
        crash_local: true,
        ext: false,
        args: "--autoplay-policy=no-user-gesture-required".into(),
        url: None,
        views: 2,
        tag: "main".into(),
        no_reputation: false,
    };
    let mut it = std::env::args().skip(1);
    while let Some(a) = it.next() {
        match a.as_str() {
            "--script" => o.script = it.next(),
            "--no-chrome" => o.no_chrome = true,
            "--job" => o.job = true,
            "--no-crash-local" => o.crash_local = false,
            "--ext" => o.ext = true,
            "--args" => o.args = it.next().unwrap_or_default(),
            "--url" => o.url = it.next(),
            "--views" => o.views = it.next().and_then(|s| s.parse().ok()).unwrap_or(2),
            "--tag" => o.tag = it.next().unwrap_or_default(),
            "--no-reputation" => o.no_reputation = true,
            x => eprintln!("unknown arg {x}"),
        }
    }
    o
}

fn content_rect(window: &Window, full: bool) -> Rect {
    let size: LogicalSize<f64> = window.inner_size().to_logical(window.scale_factor());
    let top = if full { 0.0 } else { CHROME_H };
    Rect {
        position: LogicalPosition::new(0.0, top).into(),
        size: LogicalSize::new(size.width, (size.height - top).max(0.0)).into(),
    }
}

/// Build a content (`Web`) view: no custom protocol, no IPC handler.
pub fn make_content(shared: &Rc<Shared>, url: &str, private: bool) -> View {
    let id = shared.next_id.get();
    shared.next_id.set(id + 1);
    let sh = shared.clone();
    let wv = WebViewBuilder::new()
        .with_environment(shared.env.clone())
        .with_incognito(private)
        .with_url(url)
        .with_bounds(content_rect(&shared.window, false))
        .with_visible(false)
        .with_new_window_req_handler(move |uri, features| {
            // Keep window.opener (OAuth popups, SPEC §6.6): build the popup on
            // the opener's environment with the same private flag, and hand
            // its ICoreWebView2 back via NewWindowRequestedEventArgs.NewWindow.
            info!("NewWindowRequested from v{id} uri={uri} (popup inherits private={private})");
            let pid = sh.next_id.get();
            sh.next_id.set(pid + 1);
            let built = WebViewBuilder::new()
                .with_environment(features.opener.environment.clone())
                .with_incognito(private)
                .with_bounds(content_rect(&sh.window, false))
                .build_as_child(&*sh.window);
            match built {
                Ok(pwv) => {
                    let core = pwv.webview();
                    wv2::hook_content(&core, &pwv.controller(), &sh, pid);
                    sh.pending_popups.borrow_mut().push(View {
                        id: pid,
                        wv: pwv,
                        core: core.clone(),
                        private,
                        opener: Some(id),
                    });
                    let _ = sh.proxy.send_event(UserEvent::Cmd("adopt-popups".into()));
                    NewWindowResponse::Create { webview: core }
                }
                Err(e) => {
                    warn!("popup build failed: {e}");
                    NewWindowResponse::Deny
                }
            }
        })
        .build_as_child(&*shared.window)
        .expect("content view");
    let core = wv.webview();
    wv2::hook_content(&core, &wv.controller(), shared, id);
    info!("created content view v{id} private={private} url={url}");
    View {
        id,
        wv,
        core,
        private,
        opener: None,
    }
}

struct App {
    shared: Rc<Shared>,
    chrome: Option<WebView>,
    views: Vec<View>,
    active: usize,
    overlay: bool,
    fullscreen: bool,
    job: Option<winproc::Job>,
    cpu_baseline: Vec<(u32, u64)>,
}

impl App {
    fn layout(&self) {
        let w = &self.shared.window;
        if let Some(c) = &self.chrome {
            let size: LogicalSize<f64> = w.inner_size().to_logical(w.scale_factor());
            let h = if self.overlay { size.height } else { CHROME_H };
            let _ = c.set_bounds(Rect {
                position: LogicalPosition::new(0.0, 0.0).into(),
                size: LogicalSize::new(size.width, h).into(),
            });
            let _ = c.set_visible(!self.fullscreen);
        }
        for (i, v) in self.views.iter().enumerate() {
            let _ =
                v.wv.set_bounds(content_rect(w, self.fullscreen || self.chrome.is_none()));
            let _ = v.wv.set_visible(i == self.active);
        }
        if self.overlay
            && let Some(c) = &self.chrome
        {
            // SAFETY: raising our own child HWND above its siblings.
            unsafe {
                let _ = SetWindowPos(
                    c.hwnd(),
                    Some(HWND_TOP),
                    0,
                    0,
                    0,
                    0,
                    SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE,
                );
            }
        }
    }

    fn view(&self, s: Option<&str>) -> Option<&View> {
        let i = s
            .and_then(|s| s.parse::<usize>().ok())
            .unwrap_or(self.active);
        self.views.get(i)
    }

    fn chrome_js(&self, js: &str) {
        if let Some(c) = &self.chrome {
            let _ = c.evaluate_script(js);
        }
    }

    fn total_mem(&self, label: &str) {
        let me = std::process::id();
        let (mut total, mut commit) = winproc::counters(me).unwrap_or((0, 0));
        info!(
            "MEM[{label}] pid {me} shell ws {} KiB commit {} KiB",
            total / 1024,
            commit / 1024
        );
        for (pid, kind) in wv2::process_infos(&self.shared.env) {
            let (m, c) = winproc::counters(pid).unwrap_or((0, 0));
            total += m;
            commit += c;
            info!(
                "MEM[{label}] pid {pid} {} ws {} KiB commit {} KiB",
                wv2::kind_name(kind),
                m / 1024,
                c / 1024
            );
        }
        info!(
            "MEM[{label}] TOTAL private working set {:.1} MB | private commit {:.1} MB",
            total as f64 / 1048576.0,
            commit as f64 / 1048576.0
        );
    }

    fn cmd(&mut self, line: &str) {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            return;
        }
        info!("> {line}");
        // `@/path` is shorthand for the local test server.
        let expanded = line.replace("@/", &format!("http://127.0.0.1:{}/", self.shared.port));
        let line = expanded.as_str();
        let mut p = line.splitn(3, ' ');
        let c = p.next().unwrap_or("");
        let a1 = p.next();
        let a2 = p.next();
        let rest = line.split_once(' ').map(|x| x.1).unwrap_or("");
        match c {
            "tab" => {
                if let Some(i) = a1.and_then(|s| s.parse().ok())
                    && i < self.views.len()
                {
                    self.active = i;
                    self.layout();
                    if let Some(v) = self.views.get(i) {
                        let _ = v.wv.focus();
                    }
                }
            }
            "nav" => {
                if let Some(v) = self.view(None) {
                    let _ = v.wv.load_url(rest);
                }
            }
            "navv" => {
                if let (Some(v), Some(u)) = (self.view(a1), a2) {
                    let _ = v.wv.load_url(u);
                }
            }
            "new" => {
                let private = a1 == Some("private");
                let url = if private {
                    a2.unwrap_or("about:blank")
                } else {
                    a1.unwrap_or("about:blank")
                };
                let v = make_content(&self.shared, url, private);
                self.views.push(v);
                self.active = self.views.len() - 1;
                self.layout();
            }
            "close" => {
                if let Some(i) = a1.and_then(|s| s.parse::<usize>().ok())
                    && i < self.views.len()
                {
                    self.views.remove(i);
                    self.active = self.active.min(self.views.len().saturating_sub(1));
                    self.layout();
                }
            }
            "closeid" => {
                if let Some(i) = a1
                    .and_then(|s| s.parse::<usize>().ok())
                    .and_then(|id| self.views.iter().position(|v| v.id == id))
                {
                    info!(
                        "closing view v{} (tab {i}) after window.close()",
                        self.views[i].id
                    );
                    let closed = self.views.remove(i);
                    // Return to the opener tab if it still exists.
                    self.active = closed
                        .opener
                        .and_then(|o| self.views.iter().position(|v| v.id == o))
                        .unwrap_or_else(|| self.active.min(self.views.len().saturating_sub(1)));
                    self.layout();
                }
            }
            "video" => {
                if let Some(v) = self.view(None) {
                    wv2::exec(
                        &v.core,
                        v.id,
                        "JSON.stringify([...document.querySelectorAll('video')].map(x=>({w:x.videoWidth,h:x.videoHeight,playing:!x.paused,src:(x.currentSrc||'').slice(0,40)})))",
                        |_| {},
                    );
                }
            }
            "adopt-popups" => {
                let new: Vec<View> = self.shared.pending_popups.borrow_mut().drain(..).collect();
                for v in new {
                    info!("adopted popup v{} as tab {}", v.id, self.views.len());
                    self.views.push(v);
                    self.active = self.views.len() - 1;
                }
                self.layout();
            }
            "suspend" => {
                if let Some(v) = self.view(a1) {
                    wv2::try_suspend(&v.core, v.id);
                }
            }
            "resume" => {
                if let Some(v) = self.view(a1) {
                    wv2::resume(&v.core, v.id);
                }
            }
            "state" => {
                for v in &self.views {
                    info!(
                        "[v{}] suspended={:?} audio(playing,muted)={:?} private={} visible={} {}",
                        v.id,
                        wv2::is_suspended(&v.core),
                        wv2::audio_state(&v.core),
                        v.private,
                        v.id == self.active,
                        wv2::controller_bounds(&v.wv.controller())
                    );
                }
            }
            "exec" => {
                // exec <view> <js>
                if let (Some(v), Some(js)) = (self.view(a1), a2) {
                    let core = v.core.clone();
                    let id = v.id;
                    wv2::exec(&v.core, v.id, js, move |_| {
                        info!(
                            "[v{id}] after ExecuteScript: suspended={:?}",
                            wv2::is_suspended(&core)
                        );
                    });
                }
            }
            "docscript" => {
                if let (Some(v), Some(js)) = (self.view(a1), a2) {
                    wv2::add_doc_script(&v.core, js);
                }
            }
            "memlevel" => {
                if let Some(v) = self.view(a1) {
                    info!(
                        "[v{}] SetMemoryUsageTargetLevel({:?}) -> {:?}",
                        v.id,
                        a2,
                        wv2::set_memory_target(&v.core, a2 == Some("low"))
                    );
                }
            }
            "mute" => {
                if let Some(v) = self.view(a1) {
                    info!(
                        "[v{}] SetIsMuted({:?}) -> {:?}",
                        v.id,
                        a2,
                        wv2::set_muted(&v.core, a2 == Some("on"))
                    );
                }
            }
            "click" | "rclick" => {
                if let (Some(v), Some(sel)) = (self.view(a1), a2) {
                    wv2::click(
                        &v.core,
                        v.id,
                        sel,
                        if c == "click" { "left" } else { "right" },
                    );
                }
            }
            "osclick" => {
                // Real OS click on an element: osclick <view> <selector>
                if let (Some(v), Some(sel)) = (self.view(a1), a2) {
                    let js = format!(
                        "(()=>{{const e=document.querySelector({sel:?});if(!e)return null;const r=e.getBoundingClientRect();return [r.x+r.width/2,r.y+r.height/2]}})()"
                    );
                    let hwnd = v.wv.hwnd();
                    let top = top_hwnd(&self.shared.window);
                    let scale = self.shared.window.scale_factor();
                    wv2::exec(&v.core, v.id, &js, move |res| {
                        if let Ok(p) = serde_json::from_str::<Vec<f64>>(&res) {
                            osinput::foreground(top);
                            osinput::click_client(
                                hwnd,
                                (p[0] * scale) as i32,
                                (p[1] * scale) as i32,
                            );
                        }
                    });
                }
            }
            "osclickxy" => {
                // Real OS click at window client coordinates (physical px).
                let top = top_hwnd(&self.shared.window);
                if let (Some(x), Some(y)) = (
                    a1.and_then(|s| s.parse().ok()),
                    a2.and_then(|s| s.parse().ok()),
                ) {
                    osinput::foreground(top);
                    osinput::click_client(top, x, y);
                }
            }
            "oskey" => {
                osinput::foreground(top_hwnd(&self.shared.window));
                if let Some(k) = a1.and_then(|s| s.bytes().next()) {
                    osinput::ctrl_key(k);
                }
            }
            "focus" => {
                if let Some(v) = self.view(a1) {
                    let _ = v.wv.focus();
                    wv2::move_focus(&v.wv.controller());
                }
            }
            "procs" => self.total_mem(a1.unwrap_or("now")),
            "procx" => wv2::log_extended_infos(&self.shared.env),
            "overlay" => {
                self.overlay = a1 == Some("on");
                self.layout();
                info!(
                    "overlay {} chrome {}",
                    self.overlay,
                    self.chrome
                        .as_ref()
                        .map(|c| wv2::controller_bounds(&c.controller()))
                        .unwrap_or_default()
                );
                self.chrome_js(&format!(
                    "window.setOverlay && setOverlay({}, {})",
                    self.overlay,
                    a2 != Some("noveil")
                ));
            }
            "fullscreen" => {
                self.fullscreen = a2 == Some("true");
                self.shared.window.set_fullscreen(
                    self.fullscreen
                        .then_some(tao::window::Fullscreen::Borderless(None)),
                );
                self.layout();
            }
            "accel" => {
                info!("browser command from accelerator: {rest}");
                self.chrome_js(&format!("window.flash && flash('accelerator {rest}')"));
            }
            "blocker" => match a1 {
                Some("load") => {
                    self.total_mem("before-blocker");
                    let before = winproc::private_ws(std::process::id()).unwrap_or(0);
                    let b = blocker::build();
                    let after = winproc::private_ws(std::process::id()).unwrap_or(0);
                    info!(
                        "blocker: shell private WS +{:.1} MB after building engine",
                        (after as f64 - before as f64) / 1048576.0
                    );
                    *self.shared.blocker.borrow_mut() = Some(b);
                    self.shared.blocker_on.set(true);
                }
                Some("off") => self.shared.blocker_on.set(false),
                Some("on") => self.shared.blocker_on.set(true),
                Some("reset") => {
                    if let Some(b) = self.shared.blocker.borrow_mut().as_mut() {
                        b.reset_stats();
                    }
                }
                Some("drop") => {
                    *self.shared.blocker.borrow_mut() = None;
                    self.shared.blocker_on.set(false);
                }
                _ => {
                    if let Some(b) = self.shared.blocker.borrow().as_ref() {
                        b.report();
                    }
                }
            },
            "verbose" => self.shared.verbose_requests.set(a1 == Some("on")),
            "capture" => self.shared.allow_capture.set(a1 == Some("allow")),
            "ext" => {
                let (dir, token) = make_extension(self.shared.port);
                *self.shared.ext_token.borrow_mut() = token;
                let mut seen = Vec::new();
                for v in &self.views {
                    if !seen.contains(&v.private) {
                        seen.push(v.private);
                        wv2::add_extension(&v.core, &dir, v.id);
                    }
                }
            }
            "job" => match &self.job {
                Some(j) => {
                    info!("job: shell in job = {:?}", j.contains(std::process::id()));
                    for (pid, kind) in wv2::process_infos(&self.shared.env) {
                        info!(
                            "job: pid {pid} {} in our job = {:?}",
                            wv2::kind_name(kind),
                            j.contains(pid)
                        );
                    }
                }
                None => info!("job: not started with --job"),
            },
            "cpucap" => {
                if let (Some(j), Some(pct)) = (&self.job, a1.and_then(|s| s.parse().ok())) {
                    info!("job: set CPU hard cap {pct}% -> {:?}", j.set_cpu_cap(pct));
                }
            }
            "cpu" => {
                // cpu start | cpu end : CPU time per process over the interval
                let mut pids: Vec<(u32, i32)> = wv2::process_infos(&self.shared.env);
                pids.push((std::process::id(), -1));
                let now: Vec<(u32, u64)> = pids
                    .iter()
                    .map(|(p, _)| (*p, winproc::cpu_ms(*p).unwrap_or(0)))
                    .collect();
                if a1 == Some("end") {
                    let mut total = 0;
                    for (pid, ms) in &now {
                        let b = self
                            .cpu_baseline
                            .iter()
                            .find(|(p, _)| p == pid)
                            .map(|x| x.1)
                            .unwrap_or(0);
                        let d = ms.saturating_sub(b);
                        total += d;
                        let kind = pids
                            .iter()
                            .find(|(p, _)| p == pid)
                            .map(|x| x.1)
                            .unwrap_or(-2);
                        info!(
                            "cpu: pid {pid} {} +{d} ms",
                            if kind == -1 {
                                "shell"
                            } else {
                                wv2::kind_name(kind)
                            }
                        );
                    }
                    info!("cpu: TOTAL +{total} ms");
                }
                self.cpu_baseline = now;
            }
            "eco" => {
                // eco on|off|get
                for (pid, kind) in wv2::process_infos(&self.shared.env) {
                    match a1 {
                        Some("on") | Some("off") => {
                            info!(
                                "eco: pid {pid} {} set {:?} -> {:?}",
                                wv2::kind_name(kind),
                                a1,
                                winproc::set_ecoqos(pid, a1 == Some("on"))
                            );
                        }
                        _ => info!(
                            "eco: pid {pid} {} (control,state)={:?}",
                            wv2::kind_name(kind),
                            winproc::get_ecoqos(pid)
                        ),
                    }
                }
            }
            "crashinfo" => info!(
                "FailureReportFolderPath = {}",
                wv2::failure_report_folder(&self.shared.env)
            ),
            "bounds" => {
                let w = &self.shared.window;
                info!(
                    "window scale_factor={} inner_size(physical)={:?} outer_pos={:?}",
                    w.scale_factor(),
                    w.inner_size(),
                    w.outer_position()
                );
                if let Some(c) = &self.chrome {
                    info!("chrome {}", wv2::controller_bounds(&c.controller()));
                }
                for v in &self.views {
                    info!("v{} {}", v.id, wv2::controller_bounds(&v.wv.controller()));
                }
            }
            "shot" => {
                // Screenshot of our window via PowerShell (System.Drawing), into the spike dir.
                let w = &self.shared.window;
                let pos = w.inner_position().unwrap_or_default();
                let size = w.inner_size();
                let out = logging::spike_dir()
                    .join("shots")
                    .join(format!("{}.png", a1.unwrap_or("shot")));
                let _ = std::fs::create_dir_all(out.parent().unwrap());
                let ps = format!(
                    "Add-Type -AssemblyName System.Drawing; $b=New-Object System.Drawing.Bitmap({w},{h}); $g=[System.Drawing.Graphics]::FromImage($b); $g.CopyFromScreen({x},{y},0,0,$b.Size); $b.Save('{p}')",
                    w = size.width,
                    h = size.height,
                    x = pos.x,
                    y = pos.y,
                    p = out.display()
                );
                let r = std::process::Command::new("powershell")
                    .args(["-NoProfile", "-Command", &ps])
                    .status();
                info!("shot {} -> {:?}", out.display(), r.map(|s| s.success()));
            }
            "log" => info!("NOTE: {rest}"),
            "quit" => std::process::exit(0),
            _ => warn!("unknown command: {line}"),
        }
    }
}

fn top_hwnd(w: &Window) -> windows::Win32::Foundation::HWND {
    use tao::platform::windows::WindowExtWindows;
    windows::Win32::Foundation::HWND(w.hwnd() as _)
}

/// Write the test extension (manifest + content script) with a per-run token
/// into the spike dir, outside the repo. Returns its folder.
fn make_extension(port: u16) -> (std::path::PathBuf, String) {
    let src = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("ext");
    let dir = logging::spike_dir().join("ext");
    let _ = std::fs::create_dir_all(&dir);
    for f in ["manifest.json", "content.js"] {
        std::fs::copy(src.join(f), dir.join(f)).expect("copy extension file");
    }
    let token = format!(
        "{:x}{:x}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    );
    std::fs::write(
        dir.join("token.js"),
        format!("const LWB_TOKEN = {token:?}; const LWB_PORT = {port};\n"),
    )
    .unwrap();
    info!("extension written to {} (token {token})", dir.display());
    (dir, token)
}

fn run_script(path: String, proxy: EventLoopProxy<UserEvent>) {
    std::thread::spawn(move || {
        let text = std::fs::read_to_string(&path).expect("read script");
        for line in text.lines() {
            let l = line.trim();
            if let Some(ms) = l.strip_prefix("wait ") {
                std::thread::sleep(Duration::from_millis(ms.trim().parse().unwrap_or(1000)));
            } else if !l.is_empty() && !l.starts_with('#') {
                let _ = proxy.send_event(UserEvent::Cmd(l.to_string()));
                std::thread::sleep(Duration::from_millis(50));
            }
        }
    });
}

fn main() {
    let opts = parse_opts();
    let _guard = logging::init(&opts.tag);
    info!(
        "engine-spike pid {} args {:?}",
        std::process::id(),
        std::env::args().collect::<Vec<_>>()
    );

    // Job Object first, so the WebView2 browser process (our child) inherits it.
    let job = if opts.job {
        match winproc::put_self_in_job() {
            Ok(j) => {
                info!("job: own process assigned to a KILL_ON_JOB_CLOSE job");
                Some(j)
            }
            Err(e) => {
                warn!("job: failed: {e}");
                None
            }
        }
    } else {
        None
    };

    let port = server::start();
    let event_loop = EventLoopBuilder::<UserEvent>::with_user_event().build();
    let proxy = event_loop.create_proxy();
    let window = Rc::new(
        WindowBuilder::new()
            .with_title("lwb engine spike (Phase 0b)")
            .with_inner_size(LogicalSize::new(1200.0, 800.0))
            .build(&event_loop)
            .expect("window"),
    );

    let data_dir = logging::spike_dir().join(format!("profile-{}", opts.tag));
    let env = wv2::create_env(&data_dir, &opts.args, opts.crash_local, opts.ext)
        .expect("WebView2 environment");
    info!(
        "WebView2 runtime {} | data dir {} | browser args {:?} | custom crash reporting {} | extensions {}",
        wv2::runtime_version(&env),
        data_dir.display(),
        opts.args,
        opts.crash_local,
        opts.ext
    );

    let shared = Rc::new(Shared {
        proxy: proxy.clone(),
        env,
        blocker: RefCell::new(None),
        blocker_on: Cell::new(false),
        verbose_requests: Cell::new(false),
        allow_capture: Cell::new(false),
        window: window.clone(),
        port,
        pending_popups: RefCell::new(Vec::new()),
        next_id: Cell::new(0),
        ext_token: RefCell::new(String::new()),
        no_reputation: opts.no_reputation,
    });

    let index = format!("http://127.0.0.1:{port}/pages/index.html");
    let mut views = Vec::new();
    for i in 0..opts.views {
        let url = match (i, &opts.url) {
            (0, Some(u)) => u.clone(),
            (0, None) => index.clone(),
            _ => "https://example.com/".to_string(),
        };
        views.push(make_content(&shared, &url, false));
    }

    // Chrome view: our UI, the only view with IPC and the custom protocol.
    // Created last so it starts above the content views in z-order.
    let chrome = (!opts.no_chrome).then(|| {
        let p = proxy.clone();
        WebViewBuilder::new()
            .with_environment(shared.env.clone())
            .with_transparent(true)
            .with_custom_protocol("lwb".into(), |_id, req| {
                info!("custom protocol request: {}", req.uri());
                wry::http::Response::builder()
                    .header("Content-Type", "text/html; charset=utf-8")
                    .body(std::borrow::Cow::Borrowed(CHROME_HTML.as_bytes()))
                    .unwrap()
            })
            .with_url("http://lwb.localhost/chrome.html")
            .with_ipc_handler(move |req| {
                // Second layer (SPEC §6.6): check the sender's origin.
                let origin = req.uri().to_string();
                if !origin.starts_with("http://lwb.localhost") {
                    warn!("IPC from unexpected origin {origin} dropped");
                    return;
                }
                let _ = p.send_event(UserEvent::Cmd(req.body().clone()));
            })
            .with_bounds(Rect {
                position: LogicalPosition::new(0.0, 0.0).into(),
                size: LogicalSize::new(1200.0, CHROME_H).into(),
            })
            .build_as_child(&*window)
            .expect("chrome view")
    });
    if let Some(c) = &chrome {
        wv2::hook_accelerators(&c.controller(), &shared, 99);
    }

    let mut app = App {
        shared,
        chrome,
        views,
        active: 0,
        overlay: false,
        fullscreen: false,
        job,
        cpu_baseline: Vec::new(),
    };
    app.layout();

    if let Some(s) = opts.script.clone() {
        run_script(s, proxy.clone());
    }

    event_loop.run(move |event, _, flow| {
        *flow = ControlFlow::Wait;
        match event {
            Event::UserEvent(UserEvent::Cmd(c)) => app.cmd(&c),
            Event::WindowEvent {
                event: WindowEvent::Resized(_),
                ..
            } => app.layout(),
            Event::WindowEvent {
                event: WindowEvent::ScaleFactorChanged { scale_factor, .. },
                ..
            } => {
                info!("ScaleFactorChanged -> {scale_factor}");
                app.layout();
                app.cmd("bounds");
            }
            Event::WindowEvent {
                event: WindowEvent::Moved(_),
                ..
            } => {}
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *flow = ControlFlow::Exit,
            _ => {}
        }
    });
}
