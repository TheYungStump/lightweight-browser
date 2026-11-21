//! Logging to the console and to `%LOCALAPPDATA%\lwb-spike\logs\spike-<pid>.log`.

use std::path::PathBuf;

use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

/// Root folder for everything the spike writes (profile, logs, filter lists,
/// generated extension). Deliberately outside the repo (never commit lists).
pub fn spike_dir() -> PathBuf {
    let base = std::env::var_os("LOCALAPPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(std::env::temp_dir);
    base.join("lwb-spike")
}

pub fn init(tag: &str) -> tracing_appender::non_blocking::WorkerGuard {
    let dir = spike_dir().join("logs");
    let _ = std::fs::create_dir_all(&dir);
    let file_name = format!("spike-{tag}.log");
    let appender = tracing_appender::rolling::never(&dir, &file_name);
    let (nb, guard) = tracing_appender::non_blocking(appender);
    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .with(fmt::layer().with_target(false))
        .with(
            fmt::layer()
                .with_ansi(false)
                .with_target(false)
                .with_writer(nb),
        )
        .init();
    tracing::info!("log file: {}", dir.join(file_name).display());
    guard
}
