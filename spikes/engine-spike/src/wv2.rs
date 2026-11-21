//! Direct WebView2 COM access (via `webview2-com`) for everything wry doesn't
//! expose: the shared environment, engine events, suspend/resume, memory
//! targets, process info, request interception and browser extensions.
//!
//! Every `unsafe` block here is a COM call on an interface pointer we obtained
//! from WebView2 and hold a reference to; WebView2 requires these calls on the
//! UI thread, which is the only thread this module runs on.

use std::rc::Rc;
use std::sync::mpsc;
use std::time::Instant;

use tracing::{info, warn};
use webview2_com::Microsoft::Web::WebView2::Win32::*;
use webview2_com::*;
use windows::Win32::UI::Input::KeyboardAndMouse::{GetKeyState, VK_CONTROL};
use windows::core::{BOOL, HSTRING, Interface, PWSTR};

use crate::{Shared, UserEvent};

fn pw(f: impl FnOnce(&mut PWSTR) -> windows::core::Result<()>) -> String {
    let mut p = PWSTR::null();
    match f(&mut p) {
        Ok(()) => take_pwstr(p),
        Err(_) => String::new(),
    }
}

/// Create the single shared environment (SPEC §6.4). We create it ourselves
/// (not via wry) because wry has no option for IsCustomCrashReportingEnabled,
/// and wry's default browser arguments silently disable SmartScreen.
pub fn create_env(
    data_dir: &std::path::Path,
    browser_args: &str,
    crash_local: bool,
    extensions: bool,
) -> windows::core::Result<ICoreWebView2Environment> {
    let options = CoreWebView2EnvironmentOptions::default();
    // SAFETY: setters on our own options object, before it is handed to WebView2.
    unsafe {
        options.set_additional_browser_arguments(browser_args.to_string());
        options.set_is_custom_crash_reporting_enabled(crash_local);
        options.set_are_browser_extensions_enabled(extensions);
    }
    let (tx, rx) = mpsc::channel();
    let dir = HSTRING::from(data_dir.as_os_str());
    // SAFETY: documented entry point; the completion handler owns `tx`, and we
    // pump messages until it fires (the call is async on the UI thread).
    unsafe {
        CreateCoreWebView2EnvironmentWithOptions(
            &HSTRING::new(),
            &dir,
            &ICoreWebView2EnvironmentOptions::from(options),
            &CreateCoreWebView2EnvironmentCompletedHandler::create(Box::new(move |hr, env| {
                let _ = tx.send(hr.and_then(|_| {
                    env.ok_or_else(|| {
                        windows::core::Error::from_hresult(windows::Win32::Foundation::E_POINTER)
                    })
                }));
                Ok(())
            })),
        )?;
    }
    wait_with_pump(rx)
        .map_err(|_| windows::core::Error::from_hresult(windows::Win32::Foundation::E_FAIL))?
}

fn read_stream(st: &windows::Win32::System::Com::IStream) -> String {
    let mut out = Vec::new();
    let mut buf = [0u8; 4096];
    loop {
        let mut n = 0u32;
        // SAFETY: reading into a stack buffer of the stated length.
        let hr = unsafe { st.Read(buf.as_mut_ptr() as *mut _, buf.len() as u32, Some(&mut n)) };
        if hr.is_err() || n == 0 {
            break;
        }
        out.extend_from_slice(&buf[..n as usize]);
    }
    String::from_utf8_lossy(&out).into_owned()
}

pub fn runtime_version(env: &ICoreWebView2Environment) -> String {
    // SAFETY: COM getter on a live interface (see module docs).
    pw(|p| unsafe { env.BrowserVersionString(p) })
}

pub fn failure_report_folder(env: &ICoreWebView2Environment) -> String {
    // SAFETY: COM getter on a live interface.
    unsafe {
        env.cast::<ICoreWebView2Environment11>()
            .map(|e| pw(|p| e.FailureReportFolderPath(p)))
            .unwrap_or_else(|e| format!("<Environment11 unavailable: {e}>"))
    }
}

fn host_of(uri: &str) -> &str {
    let rest = uri.split_once("://").map(|x| x.1).unwrap_or(uri);
    let end = rest.find(['/', '?', '#', ':']).unwrap_or(rest.len());
    &rest[..end]
}

fn context_name(c: COREWEBVIEW2_WEB_RESOURCE_CONTEXT) -> &'static str {
    match c {
        COREWEBVIEW2_WEB_RESOURCE_CONTEXT_DOCUMENT => "document",
        COREWEBVIEW2_WEB_RESOURCE_CONTEXT_STYLESHEET => "stylesheet",
        COREWEBVIEW2_WEB_RESOURCE_CONTEXT_IMAGE => "image",
        COREWEBVIEW2_WEB_RESOURCE_CONTEXT_MEDIA => "media",
        COREWEBVIEW2_WEB_RESOURCE_CONTEXT_FONT => "font",
        COREWEBVIEW2_WEB_RESOURCE_CONTEXT_SCRIPT => "script",
        COREWEBVIEW2_WEB_RESOURCE_CONTEXT_XML_HTTP_REQUEST => "xmlhttprequest",
        COREWEBVIEW2_WEB_RESOURCE_CONTEXT_FETCH => "fetch",
        COREWEBVIEW2_WEB_RESOURCE_CONTEXT_WEBSOCKET => "websocket",
        COREWEBVIEW2_WEB_RESOURCE_CONTEXT_PING => "ping",
        _ => "other",
    }
}

/// adblock-rust request type names.
fn adblock_type(c: &str) -> &str {
    match c {
        "fetch" => "xmlhttprequest",
        "document" => "document",
        "other" => "other",
        x => x,
    }
}

/// Register every content-view event handler the spike checks. `id` is the
/// view's index, used in log lines and commands.
pub fn hook_content(
    core: &ICoreWebView2,
    controller: &ICoreWebView2Controller,
    shared: &Rc<Shared>,
    id: usize,
) {
    let mut token = 0i64;
    if shared.no_reputation {
        // SAFETY: COM calls on a live interface.
        unsafe {
            let r = core
                .Settings()
                .and_then(|s| s.cast::<ICoreWebView2Settings8>())
                .and_then(|s8| s8.SetIsReputationCheckingRequired(false));
            info!("[v{id}] IsReputationCheckingRequired=false -> {r:?}");
        }
    }
    // SAFETY (whole function): COM calls on live interfaces on the UI thread;
    // handlers are COM objects created by webview2-com that WebView2 releases.
    unsafe {
        // Navigation: log, and block the internal host in web views (SPEC §6.6).
        let _ = core.add_NavigationStarting(
            &NavigationStartingEventHandler::create(Box::new(move |_, args| {
                let Some(args) = args else { return Ok(()) };
                let uri = pw(|p| args.Uri(p));
                let mut user = BOOL(0);
                let _ = args.IsUserInitiated(&mut user);
                let mut redirect = BOOL(0);
                let _ = args.IsRedirected(&mut redirect);
                let kind = args
                    .cast::<ICoreWebView2NavigationStartingEventArgs3>()
                    .ok()
                    .map(|a| {
                        let mut k = COREWEBVIEW2_NAVIGATION_KIND::default();
                        let _ = a.NavigationKind(&mut k);
                        k.0
                    });
                if host_of(&uri).eq_ignore_ascii_case("lwb.localhost") {
                    info!("[v{id}] NavigationStarting BLOCKED internal host: {uri}");
                    args.SetCancel(true)?;
                    return Ok(());
                }
                info!(
                    "[v{id}] NavigationStarting uri={uri} user={} redirect={} kind={kind:?}",
                    user.as_bool(),
                    redirect.as_bool()
                );
                Ok(())
            })),
            &mut token,
        );
        let _ = core.add_NavigationCompleted(
            &NavigationCompletedEventHandler::create(Box::new(move |wv, args| {
                let Some(args) = args else { return Ok(()) };
                let mut ok = BOOL(0);
                let _ = args.IsSuccess(&mut ok);
                let mut st = COREWEBVIEW2_WEB_ERROR_STATUS::default();
                let _ = args.WebErrorStatus(&mut st);
                let src = wv.map(|w| pw(|p| w.Source(p))).unwrap_or_default();
                info!(
                    "[v{id}] NavigationCompleted ok={} status={} src={src}",
                    ok.as_bool(),
                    st.0
                );
                Ok(())
            })),
            &mut token,
        );
        let _ = core.add_DocumentTitleChanged(
            &DocumentTitleChangedEventHandler::create(Box::new(move |wv, _| {
                if let Some(wv) = wv {
                    info!("[v{id}] Title: {}", pw(|p| wv.DocumentTitle(p)));
                }
                Ok(())
            })),
            &mut token,
        );
        let _ = core.add_WebMessageReceived(
            &WebMessageReceivedEventHandler::create(Box::new(move |_, args| {
                let Some(args) = args else { return Ok(()) };
                let src = pw(|p| args.Source(p));
                let msg = pw(|p| args.TryGetWebMessageAsString(p));
                info!("[v{id}] WebMessage from {src}: {msg}");
                Ok(())
            })),
            &mut token,
        );

        // Request interception: worker source kinds, POST detection, ad blocking.
        match core.cast::<ICoreWebView2_22>() {
            Ok(c22) => {
                let r = c22.AddWebResourceRequestedFilterWithRequestSourceKinds(
                    &HSTRING::from("*"),
                    COREWEBVIEW2_WEB_RESOURCE_CONTEXT_ALL,
                    COREWEBVIEW2_WEB_RESOURCE_REQUEST_SOURCE_KINDS_ALL,
                );
                info!("[v{id}] AddWebResourceRequestedFilterWithRequestSourceKinds(ALL) -> {r:?}");
            }
            Err(e) => warn!("[v{id}] ICoreWebView2_22 unavailable: {e}"),
        }
        let sh = shared.clone();
        let _ = core.add_WebResourceRequested(
            &WebResourceRequestedEventHandler::create(Box::new(move |wv, args| {
                let t = Instant::now();
                let Some(args) = args else { return Ok(()) };
                let req = args.Request()?;
                let uri = pw(|p| req.Uri(p));
                let mut ctx = COREWEBVIEW2_WEB_RESOURCE_CONTEXT::default();
                let _ = args.ResourceContext(&mut ctx);
                let cname = context_name(ctx);
                let src_kind = args
                    .cast::<ICoreWebView2WebResourceRequestedEventArgs2>()
                    .ok()
                    .map(|a| {
                        let mut k = COREWEBVIEW2_WEB_RESOURCE_REQUEST_SOURCE_KINDS::default();
                        let _ = a.RequestedSourceKind(&mut k);
                        k.0
                    })
                    .unwrap_or(0);
                if ctx == COREWEBVIEW2_WEB_RESOURCE_CONTEXT_DOCUMENT || sh.verbose_requests.get() {
                    let method = pw(|p| req.Method(p));
                    info!("[v{id}] WebResourceRequested {method} {cname} srcKind={src_kind} {uri}");
                } else if src_kind != 1 {
                    info!(
                        "[v{id}] WebResourceRequested from WORKER srcKind={src_kind} {cname} {uri}"
                    );
                }
                if host_of(&uri).eq_ignore_ascii_case("lwb-channel.invalid") {
                    // Isolated-world channel test: the extension posts here; we answer
                    // locally (never hits the network) and check the per-run token.
                    let body = req
                        .Content()
                        .ok()
                        .map(|st| read_stream(&st))
                        .unwrap_or_default();
                    let ok = !sh.ext_token.borrow().is_empty()
                        && body.contains(&format!("\"token\":\"{}\"", sh.ext_token.borrow()));
                    info!("[v{id}] CHANNEL message token_ok={ok} body={body}");
                    let resp = sh.env.CreateWebResourceResponse(
                        None,
                        204,
                        &HSTRING::from("No Content"),
                        &HSTRING::new(),
                    )?;
                    args.SetResponse(&resp)?;
                    return Ok(());
                }
                let internal = host_of(&uri).eq_ignore_ascii_case("lwb.localhost");
                let mut block = internal;
                if !internal
                    && sh.blocker_on.get()
                    && let Some(b) = sh.blocker.borrow_mut().as_mut()
                {
                    let page = wv.map(|w| pw(|p| w.Source(p))).unwrap_or_default();
                    block = b.check(&uri, &page, adblock_type(cname));
                }
                if block {
                    let resp = sh.env.CreateWebResourceResponse(
                        None,
                        403,
                        &HSTRING::from("Blocked"),
                        &HSTRING::new(),
                    )?;
                    args.SetResponse(&resp)?;
                    if internal {
                        info!("[v{id}] request to internal host BLOCKED: {uri}");
                    }
                }
                if !internal
                    && sh.blocker_on.get()
                    && let Some(b) = sh.blocker.borrow_mut().as_mut()
                {
                    b.handler_ns.push(t.elapsed().as_nanos() as u64);
                }
                Ok(())
            })),
            &mut token,
        );

        // Audio.
        if let Ok(c8) = core.cast::<ICoreWebView2_8>() {
            let _ = c8.add_IsDocumentPlayingAudioChanged(
                &IsDocumentPlayingAudioChangedEventHandler::create(Box::new(move |wv, _| {
                    if let Some(c8) = wv.and_then(|w| w.cast::<ICoreWebView2_8>().ok()) {
                        let (mut a, mut m) = (BOOL(0), BOOL(0));
                        let _ = c8.IsDocumentPlayingAudio(&mut a);
                        let _ = c8.IsMuted(&mut m);
                        info!(
                            "[v{id}] IsDocumentPlayingAudioChanged playing={} muted={}",
                            a.as_bool(),
                            m.as_bool()
                        );
                    }
                    Ok(())
                })),
                &mut token,
            );
            let _ = c8.add_IsMutedChanged(
                &IsMutedChangedEventHandler::create(Box::new(move |wv, _| {
                    if let Some(c8) = wv.and_then(|w| w.cast::<ICoreWebView2_8>().ok()) {
                        let (mut a, mut m) = (BOOL(0), BOOL(0));
                        let _ = c8.IsDocumentPlayingAudio(&mut a);
                        let _ = c8.IsMuted(&mut m);
                        info!(
                            "[v{id}] IsMutedChanged muted={} playing={}",
                            m.as_bool(),
                            a.as_bool()
                        );
                    }
                    Ok(())
                })),
                &mut token,
            );
        }

        // Fullscreen.
        let sh = shared.clone();
        let _ = core.add_ContainsFullScreenElementChanged(
            &ContainsFullScreenElementChangedEventHandler::create(Box::new(move |wv, _| {
                if let Some(wv) = wv {
                    let mut fs = BOOL(0);
                    let _ = wv.ContainsFullScreenElement(&mut fs);
                    info!(
                        "[v{id}] ContainsFullScreenElementChanged fullscreen={}",
                        fs.as_bool()
                    );
                    let _ = sh
                        .proxy
                        .send_event(UserEvent::Cmd(format!("fullscreen {id} {}", fs.as_bool())));
                }
                Ok(())
            })),
            &mut token,
        );

        // Permissions: always ours (SPEC §6.6). The spike allows, to let tests proceed.
        let _ = core.add_PermissionRequested(
            &PermissionRequestedEventHandler::create(Box::new(move |_, args| {
                let Some(args) = args else { return Ok(()) };
                let uri = pw(|p| args.Uri(p));
                let mut kind = COREWEBVIEW2_PERMISSION_KIND::default();
                let _ = args.PermissionKind(&mut kind);
                let mut user = BOOL(0);
                let _ = args.IsUserInitiated(&mut user);
                info!(
                    "[v{id}] PermissionRequested kind={} user={} uri={uri} -> ALLOW (spike)",
                    kind.0,
                    user.as_bool()
                );
                if let Ok(a3) = args.cast::<ICoreWebView2PermissionRequestedEventArgs3>() {
                    let _ = a3.SetSavesInProfile(false);
                }
                args.SetState(COREWEBVIEW2_PERMISSION_STATE_ALLOW)?;
                Ok(())
            })),
            &mut token,
        );

        if let Ok(c14) = core.cast::<ICoreWebView2_14>() {
            let _ = c14.add_ServerCertificateErrorDetected(
                &ServerCertificateErrorDetectedEventHandler::create(Box::new(move |_, args| {
                    let Some(args) = args else { return Ok(()) };
                    let uri = pw(|p| args.RequestUri(p));
                    let mut st = COREWEBVIEW2_WEB_ERROR_STATUS::default();
                    let _ = args.ErrorStatus(&mut st);
                    info!(
                        "[v{id}] ServerCertificateErrorDetected status={} uri={uri} -> CANCEL",
                        st.0
                    );
                    args.SetAction(COREWEBVIEW2_SERVER_CERTIFICATE_ERROR_ACTION_CANCEL)?;
                    Ok(())
                })),
                &mut token,
            );
        }

        if let Ok(c10) = core.cast::<ICoreWebView2_10>() {
            let _ = c10.add_BasicAuthenticationRequested(
                &BasicAuthenticationRequestedEventHandler::create(Box::new(move |_, args| {
                    let Some(args) = args else { return Ok(()) };
                    let uri = pw(|p| args.Uri(p));
                    let ch = pw(|p| args.Challenge(p));
                    info!("[v{id}] BasicAuthenticationRequested uri={uri} challenge={ch} -> answering spike/spike");
                    let r = args.Response()?;
                    r.SetUserName(&HSTRING::from("spike"))?;
                    r.SetPassword(&HSTRING::from("spike"))?;
                    Ok(())
                })),
                &mut token,
            );
        }

        if let Ok(c11) = core.cast::<ICoreWebView2_11>() {
            let _ = c11.add_ContextMenuRequested(
                &ContextMenuRequestedEventHandler::create(Box::new(move |_, args| {
                    let Some(args) = args else { return Ok(()) };
                    let mut pt = windows::Win32::Foundation::POINT::default();
                    let _ = args.Location(&mut pt);
                    let target = args.ContextMenuTarget()?;
                    let mut kind = COREWEBVIEW2_CONTEXT_MENU_TARGET_KIND::default();
                    let _ = target.Kind(&mut kind);
                    let mut has_link = BOOL(0);
                    let _ = target.HasLinkUri(&mut has_link);
                    let link = if has_link.as_bool() { pw(|p| target.LinkUri(p)) } else { String::new() };
                    info!("[v{id}] ContextMenuRequested at ({},{}) kind={} link={link} -> handled (our menu)", pt.x, pt.y, kind.0);
                    args.SetHandled(true)?;
                    Ok(())
                })),
                &mut token,
            );
        }

        if let Ok(c27) = core.cast::<ICoreWebView2_27>() {
            let sh = shared.clone();
            let _ = c27.add_ScreenCaptureStarting(
                &ScreenCaptureStartingEventHandler::create(Box::new(move |_, args| {
                    let Some(args) = args else { return Ok(()) };
                    let allow = sh.allow_capture.get();
                    info!(
                        "[v{id}] ScreenCaptureStarting -> {}",
                        if allow {
                            "allow (picker shows)"
                        } else {
                            "CANCEL"
                        }
                    );
                    args.SetCancel(!allow)?;
                    Ok(())
                })),
                &mut token,
            );
        } else {
            warn!("[v{id}] ICoreWebView2_27 (ScreenCaptureStarting) unavailable");
        }

        // window.close() from a page (OAuth popups close themselves when done).
        let sh = shared.clone();
        let _ = core.add_WindowCloseRequested(
            &WindowCloseRequestedEventHandler::create(Box::new(move |_, _| {
                info!("[v{id}] WindowCloseRequested");
                let _ = sh.proxy.send_event(UserEvent::Cmd(format!("closeid {id}")));
                Ok(())
            })),
            &mut token,
        );

        // DRM probe (spike only, main world): log which EME key system a site
        // asks for and gets.
        add_doc_script(
            core,
            r#"(()=>{const o=navigator.requestMediaKeySystemAccess&&navigator.requestMediaKeySystemAccess.bind(navigator);if(!o)return;navigator.requestMediaKeySystemAccess=(ks,cfg)=>o(ks,cfg).then(a=>{try{chrome.webview.postMessage('EME granted '+ks+' robustness='+JSON.stringify((a.getConfiguration().videoCapabilities||[]).map(c=>c.robustness)))}catch(e){}return a},e=>{try{chrome.webview.postMessage('EME denied '+ks+' '+e.name)}catch(_){}throw e})})()"#,
        );

        let _ = core.add_ProcessFailed(
            &ProcessFailedEventHandler::create(Box::new(move |_, args| {
                let Some(args) = args else { return Ok(()) };
                let mut k = COREWEBVIEW2_PROCESS_FAILED_KIND::default();
                let _ = args.ProcessFailedKind(&mut k);
                info!("[v{id}] ProcessFailed kind={}", k.0);
                Ok(())
            })),
            &mut token,
        );

        hook_accelerators(controller, shared, id);
    }
}

/// Native accelerators that work while the page has focus (SPEC §8.4).
pub fn hook_accelerators(controller: &ICoreWebView2Controller, shared: &Rc<Shared>, id: usize) {
    let sh = shared.clone();
    let mut token = 0i64;
    // SAFETY: COM registration on a live controller, UI thread.
    unsafe {
        let _ = controller.add_AcceleratorKeyPressed(
            &AcceleratorKeyPressedEventHandler::create(Box::new(move |_, args| {
                let Some(args) = args else { return Ok(()) };
                let mut kind = COREWEBVIEW2_KEY_EVENT_KIND::default();
                let _ = args.KeyEventKind(&mut kind);
                let mut vk = 0u32;
                let _ = args.VirtualKey(&mut vk);
                let ctrl = GetKeyState(VK_CONTROL.0 as i32) < 0;
                let down = kind == COREWEBVIEW2_KEY_EVENT_KIND_KEY_DOWN;
                if ctrl && down && matches!(vk as u8, b'T' | b'L' | b'W') {
                    info!(
                        "[v{id}] AcceleratorKeyPressed Ctrl+{} -> handled by browser",
                        vk as u8 as char
                    );
                    args.SetHandled(true)?;
                    let _ = sh.proxy.send_event(UserEvent::Cmd(format!(
                        "accel ctrl+{}",
                        (vk as u8 as char).to_ascii_lowercase()
                    )));
                } else if down && ctrl {
                    info!("[v{id}] AcceleratorKeyPressed Ctrl+vk={vk:#x} (passed to page)");
                }
                Ok(())
            })),
            &mut token,
        );
    }
}

pub fn try_suspend(core: &ICoreWebView2, id: usize) {
    let t = Instant::now();
    // SAFETY: COM call on a live interface, UI thread.
    unsafe {
        match core.cast::<ICoreWebView2_3>() {
            Ok(c3) => {
                let r = c3.TrySuspend(&TrySuspendCompletedHandler::create(Box::new(
                    move |hr, ok| {
                        info!(
                            "[v{id}] TrySuspend completed hr={hr:?} success={ok} after {:?}",
                            t.elapsed()
                        );
                        Ok(())
                    },
                )));
                if let Err(e) = r {
                    info!("[v{id}] TrySuspend call failed: {e}");
                }
            }
            Err(e) => warn!("[v{id}] ICoreWebView2_3 unavailable: {e}"),
        }
    }
}

pub fn resume(core: &ICoreWebView2, id: usize) {
    // SAFETY: COM call on a live interface, UI thread.
    unsafe {
        if let Ok(c3) = core.cast::<ICoreWebView2_3>() {
            info!("[v{id}] Resume -> {:?}", c3.Resume());
        }
    }
}

pub fn is_suspended(core: &ICoreWebView2) -> Option<bool> {
    // SAFETY: COM getter on a live interface.
    unsafe {
        let c3 = core.cast::<ICoreWebView2_3>().ok()?;
        let mut b = BOOL(0);
        c3.IsSuspended(&mut b).ok()?;
        Some(b.as_bool())
    }
}

pub fn set_muted(core: &ICoreWebView2, m: bool) -> windows::core::Result<()> {
    // SAFETY: COM call on a live interface.
    unsafe { core.cast::<ICoreWebView2_8>()?.SetIsMuted(m) }
}

pub fn audio_state(core: &ICoreWebView2) -> Option<(bool, bool)> {
    // SAFETY: COM getters on a live interface.
    unsafe {
        let c8 = core.cast::<ICoreWebView2_8>().ok()?;
        let (mut a, mut m) = (BOOL(0), BOOL(0));
        c8.IsDocumentPlayingAudio(&mut a).ok()?;
        c8.IsMuted(&mut m).ok()?;
        Some((a.as_bool(), m.as_bool()))
    }
}

pub fn set_memory_target(core: &ICoreWebView2, low: bool) -> windows::core::Result<()> {
    let level = if low {
        COREWEBVIEW2_MEMORY_USAGE_TARGET_LEVEL_LOW
    } else {
        COREWEBVIEW2_MEMORY_USAGE_TARGET_LEVEL_NORMAL
    };
    // SAFETY: COM call on a live interface.
    unsafe {
        core.cast::<ICoreWebView2_19>()?
            .SetMemoryUsageTargetLevel(level)
    }
}

/// Run a script and log its JSON result.
pub fn exec(core: &ICoreWebView2, id: usize, js: &str, then: impl FnOnce(String) + 'static) {
    let t = Instant::now();
    // SAFETY: COM call on a live interface; handler owned by WebView2.
    unsafe {
        let r = core.ExecuteScript(
            &HSTRING::from(js),
            &ExecuteScriptCompletedHandler::create(Box::new(move |hr, result| {
                info!(
                    "[v{id}] ExecuteScript done hr={hr:?} in {:?} result={result}",
                    t.elapsed()
                );
                then(result);
                Ok(())
            })),
        );
        if let Err(e) = r {
            info!("[v{id}] ExecuteScript call failed: {e}");
        }
    }
}

pub fn add_doc_script(core: &ICoreWebView2, js: &str) {
    // SAFETY: COM call on a live interface.
    unsafe {
        let _ = core.AddScriptToExecuteOnDocumentCreated(
            &HSTRING::from(js),
            &AddScriptToExecuteOnDocumentCreatedCompletedHandler::create(Box::new(|hr, _id| {
                info!("AddScriptToExecuteOnDocumentCreated -> {hr:?}");
                Ok(())
            })),
        );
    }
}

/// DevTools-protocol call; used to synthesize trusted mouse clicks in tests.
pub fn cdp(core: &ICoreWebView2, method: &str, params: &str) {
    let m = method.to_string();
    // SAFETY: COM call on a live interface.
    unsafe {
        let _ = core.CallDevToolsProtocolMethod(
            &HSTRING::from(method),
            &HSTRING::from(params),
            &CallDevToolsProtocolMethodCompletedHandler::create(Box::new(move |hr, r| {
                if hr.is_err() {
                    info!("CDP {m} -> {hr:?} {r}");
                }
                Ok(())
            })),
        );
    }
}

/// Click the centre of the element matched by `selector` with a real
/// (trusted, user-activation-granting) DevTools input event.
pub fn click(core: &ICoreWebView2, id: usize, selector: &str, button: &str) {
    let js = format!(
        "(()=>{{const e=document.querySelector({sel:?});if(!e)return null;const r=e.getBoundingClientRect();return [r.x+r.width/2,r.y+r.height/2]}})()",
        sel = selector
    );
    let core2 = core.clone();
    let button = button.to_string();
    exec(core, id, &js, move |res| {
        let Ok(v) = serde_json::from_str::<Vec<f64>>(&res) else {
            info!("[v{id}] click: element not found");
            return;
        };
        for t in ["mousePressed", "mouseReleased"] {
            cdp(
                &core2,
                "Input.dispatchMouseEvent",
                &format!(
                    r#"{{"type":"{t}","x":{},"y":{},"button":"{button}","clickCount":1}}"#,
                    v[0], v[1]
                ),
            );
        }
    });
}

pub fn process_infos(env: &ICoreWebView2Environment) -> Vec<(u32, i32)> {
    let mut out = Vec::new();
    // SAFETY: COM getters on live interfaces.
    unsafe {
        let Ok(e8) = env.cast::<ICoreWebView2Environment8>() else {
            return out;
        };
        let Ok(col) = e8.GetProcessInfos() else {
            return out;
        };
        let mut n = 0u32;
        let _ = col.Count(&mut n);
        for i in 0..n {
            if let Ok(pi) = col.GetValueAtIndex(i) {
                let mut pid = 0i32;
                let mut kind = COREWEBVIEW2_PROCESS_KIND::default();
                let _ = pi.ProcessId(&mut pid);
                let _ = pi.Kind(&mut kind);
                out.push((pid as u32, kind.0));
            }
        }
    }
    out
}

pub fn kind_name(k: i32) -> &'static str {
    match k {
        0 => "browser",
        1 => "renderer",
        2 => "utility",
        3 => "sandbox-helper",
        4 => "gpu",
        5 => "ppapi-plugin",
        6 => "ppapi-broker",
        _ => "?",
    }
}

/// Log GetProcessExtendedInfos: each process with the frames it hosts.
pub fn log_extended_infos(env: &ICoreWebView2Environment) {
    // SAFETY: COM calls on live interfaces; the handler runs on the UI thread.
    unsafe {
        let Ok(e13) = env.cast::<ICoreWebView2Environment13>() else {
            warn!("ICoreWebView2Environment13 unavailable");
            return;
        };
        let _ = e13.GetProcessExtendedInfos(&GetProcessExtendedInfosCompletedHandler::create(
            Box::new(|hr, col| {
                info!("GetProcessExtendedInfos hr={hr:?}");
                let Some(col) = col else { return Ok(()) };
                let mut n = 0u32;
                col.Count(&mut n)?;
                for i in 0..n {
                    let ext = col.GetValueAtIndex(i)?;
                    let pi = ext.ProcessInfo()?;
                    let mut pid = 0i32;
                    let mut kind = COREWEBVIEW2_PROCESS_KIND::default();
                    pi.ProcessId(&mut pid)?;
                    pi.Kind(&mut kind)?;
                    let mem = crate::winproc::private_ws(pid as u32).unwrap_or(0) / 1024;
                    let mut frames = Vec::new();
                    if let Ok(fc) = ext.AssociatedFrameInfos() {
                        let it = fc.GetIterator()?;
                        let mut has = BOOL(0);
                        it.HasCurrent(&mut has)?;
                        while has.as_bool() {
                            let fi = it.GetCurrent()?;
                            let src = pw(|p| fi.Source(p));
                            let (fid, fkind) = fi
                                .cast::<ICoreWebView2FrameInfo2>()
                                .ok()
                                .map(|f2| {
                                    let mut id = 0u32;
                                    let mut k = COREWEBVIEW2_FRAME_KIND::default();
                                    let _ = f2.FrameId(&mut id);
                                    let _ = f2.FrameKind(&mut k);
                                    (id, k.0)
                                })
                                .unwrap_or((0, -1));
                            frames.push(format!("frame#{fid}(kind {fkind}) {src}"));
                            it.MoveNext(&mut has)?;
                        }
                    }
                    info!(
                        "  pid {pid} {} {mem} KiB frames={frames:?}",
                        kind_name(kind.0)
                    );
                }
                Ok(())
            }),
        ));
    }
}

/// Load an unpacked extension into the view's profile.
pub fn add_extension(core: &ICoreWebView2, path: &std::path::Path, id: usize) {
    // SAFETY: COM calls on live interfaces.
    unsafe {
        let prof = match core.cast::<ICoreWebView2_13>().and_then(|c| c.Profile()) {
            Ok(p) => p,
            Err(e) => {
                warn!("[v{id}] Profile unavailable: {e}");
                return;
            }
        };
        let Ok(p7) = prof.cast::<ICoreWebView2Profile7>() else {
            warn!("[v{id}] ICoreWebView2Profile7 unavailable");
            return;
        };
        let r = p7.AddBrowserExtension(
            &HSTRING::from(path.as_os_str()),
            &ProfileAddBrowserExtensionCompletedHandler::create(Box::new(move |hr, ext| {
                let (eid, name) = ext
                    .map(|e| (pw(|p| e.Id(p)), pw(|p| e.Name(p))))
                    .unwrap_or_default();
                info!("[v{id}] AddBrowserExtension hr={hr:?} id={eid} name={name}");
                Ok(())
            })),
        );
        if let Err(e) = r {
            warn!("[v{id}] AddBrowserExtension call failed: {e}");
        }
    }
}

pub fn controller_bounds(c: &ICoreWebView2Controller) -> String {
    // SAFETY: COM getters on a live controller.
    unsafe {
        let mut r = windows::Win32::Foundation::RECT::default();
        let _ = c.Bounds(&mut r);
        let scale = c
            .cast::<ICoreWebView2Controller3>()
            .ok()
            .map(|c3| {
                let mut s = 0f64;
                let _ = c3.RasterizationScale(&mut s);
                s
            })
            .unwrap_or(0.0);
        format!(
            "bounds=({},{})-({},{}) rasterScale={scale}",
            r.left, r.top, r.right, r.bottom
        )
    }
}

pub fn move_focus(c: &ICoreWebView2Controller) {
    // SAFETY: COM call on a live controller.
    unsafe {
        let _ = c.MoveFocus(COREWEBVIEW2_MOVE_FOCUS_REASON_PROGRAMMATIC);
    }
}
