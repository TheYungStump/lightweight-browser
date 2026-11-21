//! adblock-rust with EasyList + EasyPrivacy, downloaded at runtime into
//! `%LOCALAPPDATA%\lwb-spike\lists` (outside the repo; lists are never committed).

use std::time::Instant;

use adblock::Engine;
use adblock::lists::{FilterSet, ParseOptions};
use adblock::request::Request;
use tracing::info;

use crate::logging::spike_dir;

const LISTS: &[(&str, &str)] = &[
    (
        "easylist.filterlist",
        "https://easylist.to/easylist/easylist.txt",
    ),
    (
        "easyprivacy.filterlist",
        "https://easylist.to/easylist/easyprivacy.txt",
    ),
];

fn fetch_lists() -> Vec<String> {
    let dir = spike_dir().join("lists");
    let _ = std::fs::create_dir_all(&dir);
    LISTS
        .iter()
        .map(|(file, url)| {
            let path = dir.join(file);
            if let Ok(s) = std::fs::read_to_string(&path) {
                return s;
            }
            info!("blocker: downloading {url}");
            let body = ureq::get(*url)
                .call()
                .and_then(|mut r| r.body_mut().read_to_string())
                .expect("download filter list");
            std::fs::write(&path, &body).expect("cache filter list");
            body
        })
        .collect()
}

pub struct Blocker {
    pub engine: Engine,
    /// Full-handler and engine-only timings, in nanoseconds.
    pub handler_ns: Vec<u64>,
    pub engine_ns: Vec<u64>,
    pub checked: u64,
    pub blocked: u64,
}

pub fn build() -> Blocker {
    let lists = fetch_lists();
    let t = Instant::now();
    let mut set = FilterSet::new(false);
    let mut n = 0;
    for l in lists {
        n += l.lines().count();
        set.add_filter_list(l, ParseOptions::default());
    }
    let engine = Engine::new_with_filter_set(set);
    info!("blocker: engine built from {n} lines in {:?}", t.elapsed());
    Blocker {
        engine,
        handler_ns: Vec::new(),
        engine_ns: Vec::new(),
        checked: 0,
        blocked: 0,
    }
}

impl Blocker {
    /// Returns true if the request should be blocked.
    pub fn check(&mut self, url: &str, source: &str, kind: &str) -> bool {
        let t = Instant::now();
        let blocked = match Request::new(url, source, kind, "GET") {
            Ok(req) => self.engine.check_network_request(&req).should_block(),
            Err(_) => false,
        };
        self.engine_ns.push(t.elapsed().as_nanos() as u64);
        self.checked += 1;
        if blocked {
            self.blocked += 1;
        }
        blocked
    }

    pub fn report(&self) {
        info!(
            "blocker: checked={} blocked={} handler p50={} p99={} max={} | engine p50={} p99={} max={} (µs)",
            self.checked,
            self.blocked,
            pct(&self.handler_ns, 50.0),
            pct(&self.handler_ns, 99.0),
            pct(&self.handler_ns, 100.0),
            pct(&self.engine_ns, 50.0),
            pct(&self.engine_ns, 99.0),
            pct(&self.engine_ns, 100.0),
        );
    }

    pub fn reset_stats(&mut self) {
        self.handler_ns.clear();
        self.engine_ns.clear();
        self.checked = 0;
        self.blocked = 0;
    }
}

fn pct(v: &[u64], p: f64) -> String {
    if v.is_empty() {
        return "-".into();
    }
    let mut s = v.to_vec();
    s.sort_unstable();
    let i = (((p / 100.0) * (s.len() as f64)).ceil() as usize).clamp(1, s.len()) - 1;
    format!("{:.1}", s[i] as f64 / 1000.0)
}
