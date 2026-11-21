//! Tiny HTTP/1.1 test server on 127.0.0.1 (loopback is a secure context, so
//! service workers, getUserMedia and EME work). Serves `pages/` from disk plus a
//! few endpoints:
//! - `POST /post-echo`   echoes the form body (POST-detection test)
//! - `GET  /auth`        HTTP Basic auth, user `spike` / password `spike`
//! - `GET  /ping/<what>` logs the hit and returns 204 (worker requests, heartbeat)

use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::PathBuf;

use tracing::{info, warn};

pub fn start() -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind test server");
    let port = listener.local_addr().unwrap().port();
    std::thread::spawn(move || {
        for stream in listener.incoming().flatten() {
            std::thread::spawn(move || {
                if let Err(e) = handle(stream) {
                    warn!("server: {e}");
                }
            });
        }
    });
    info!("test server on http://127.0.0.1:{port}/pages/index.html");
    port
}

fn pages_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("pages")
}

fn handle(stream: TcpStream) -> std::io::Result<()> {
    let mut reader = BufReader::new(stream.try_clone()?);
    loop {
        let mut line = String::new();
        if reader.read_line(&mut line)? == 0 {
            return Ok(());
        }
        let mut parts = line.split_whitespace();
        let method = parts.next().unwrap_or("").to_string();
        let path = parts.next().unwrap_or("/").to_string();
        let mut content_length = 0usize;
        let mut auth = None;
        loop {
            let mut h = String::new();
            reader.read_line(&mut h)?;
            let h = h.trim_end();
            if h.is_empty() {
                break;
            }
            if let Some((k, v)) = h.split_once(':') {
                let k = k.trim().to_ascii_lowercase();
                let v = v.trim().to_string();
                if k == "content-length" {
                    content_length = v.parse().unwrap_or(0);
                } else if k == "authorization" {
                    auth = Some(v);
                }
            }
        }
        let mut body = vec![0u8; content_length];
        reader.read_exact(&mut body)?;
        let (status, ctype, extra, payload) = route(&method, &path, auth.as_deref(), &body);
        let mut out = stream.try_clone()?;
        write!(
            out,
            "HTTP/1.1 {status}\r\nContent-Type: {ctype}\r\nContent-Length: {}\r\nCache-Control: no-store\r\n{extra}\r\n",
            payload.len()
        )?;
        out.write_all(&payload)?;
        out.flush()?;
    }
}

type Resp = (&'static str, &'static str, String, Vec<u8>);

fn route(method: &str, path: &str, auth: Option<&str>, body: &[u8]) -> Resp {
    let clean = path.split('?').next().unwrap_or("/");
    if let Some(what) = clean.strip_prefix("/ping/") {
        info!("server: ping {what} ({path})");
        return ("204 No Content", "text/plain", String::new(), Vec::new());
    }
    if clean == "/post-echo" {
        let text = String::from_utf8_lossy(body);
        info!("server: {method} /post-echo body={text}");
        let html = format!(
            "<!doctype html><title>post-echo</title><h1>{method} received</h1><pre>{text}</pre>"
        );
        return (
            "200 OK",
            "text/html; charset=utf-8",
            String::new(),
            html.into_bytes(),
        );
    }
    if clean == "/auth" {
        // "spike:spike" in base64.
        if auth == Some("Basic c3Bpa2U6c3Bpa2U=") {
            info!("server: /auth accepted");
            let html = "<!doctype html><title>auth ok</title><h1>Basic auth OK</h1>";
            return ("200 OK", "text/html", String::new(), html.into());
        }
        info!("server: /auth challenge (got {auth:?})");
        return (
            "401 Unauthorized",
            "text/html",
            "WWW-Authenticate: Basic realm=\"spike\"\r\n".into(),
            b"auth required".to_vec(),
        );
    }
    if let Some(rel) = clean.strip_prefix("/pages/") {
        if rel.contains("..") {
            return ("400 Bad Request", "text/plain", String::new(), Vec::new());
        }
        let p = pages_dir().join(rel);
        if let Ok(data) = std::fs::read(&p) {
            let ctype = match p.extension().and_then(|e| e.to_str()) {
                Some("html") => "text/html; charset=utf-8",
                Some("js") => "text/javascript",
                Some("css") => "text/css",
                _ => "application/octet-stream",
            };
            // Service worker scope: allow the SW at /pages/ to control /pages/.
            return (
                "200 OK",
                ctype,
                "Service-Worker-Allowed: /\r\n".into(),
                data,
            );
        }
    }
    (
        "404 Not Found",
        "text/plain",
        String::new(),
        b"not found".to_vec(),
    )
}
