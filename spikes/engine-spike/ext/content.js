// Content script of the test extension. token.js (generated per run, outside
// the repo) defines LWB_TOKEN and LWB_PORT.
window.__extMarker = 'ext';
document.documentElement.setAttribute('data-ext', 'loaded');

function probe() {
  const r = {
    ext_sees_pageMarker: typeof window.__pageMarker,
    ext_has_chrome_webview: typeof (window.chrome && window.chrome.webview),
    ext_has_chrome_runtime: typeof (window.chrome && chrome.runtime && chrome.runtime.id),
    url: location.href,
    t: Date.now(),
  };
  // Channel 1: chrome.webview.postMessage, if it exists in this world.
  try {
    if (window.chrome && chrome.webview) chrome.webview.postMessage(JSON.stringify({ from: 'ext', r }));
  } catch (e) { r.webview_err = String(e); }
  // Channel 2: a simple (no-preflight) POST to a reserved host that the host
  // app answers itself in WebResourceRequested. The per-run token is in the
  // BODY: no-cors strips custom headers, and the page can't read the body.
  fetch('https://lwb-channel.invalid/msg', {
    method: 'POST', mode: 'no-cors', headers: { 'Content-Type': 'text/plain' },
    body: JSON.stringify({ token: LWB_TOKEN, r }),
  }).catch(() => {});
}
if (document.readyState === 'loading') document.addEventListener('DOMContentLoaded', () => setTimeout(probe, 500));
else setTimeout(probe, 500);
