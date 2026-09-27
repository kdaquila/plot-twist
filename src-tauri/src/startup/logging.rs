//! JSON-lines logs in `%LOCALAPPDATA%\plot-twist\logs\`, rotated daily, 7 files kept.

use tracing_appender::non_blocking::WorkerGuard;
use tracing_appender::rolling::{Builder, Rotation};
use tracing_subscriber::EnvFilter;

use super::paths;

/// Starts logging. Keep the returned guard alive until exit so logs are flushed.
pub fn init() -> Option<WorkerGuard> {
    let filter =
        EnvFilter::try_from_env("PLOT_TWIST_LOG").unwrap_or_else(|_| EnvFilter::new("info"));
    let appender = Builder::new()
        .rotation(Rotation::DAILY)
        .filename_prefix("plot-twist")
        .filename_suffix("log")
        .max_log_files(7)
        .build(paths::logs_dir());
    match appender {
        Ok(appender) => {
            let (writer, guard) = tracing_appender::non_blocking(appender);
            tracing_subscriber::fmt()
                .json()
                .with_env_filter(filter)
                .with_writer(writer)
                .init();
            Some(guard)
        }
        Err(error) => {
            tracing_subscriber::fmt().with_env_filter(filter).init();
            tracing::warn!(%error, "could not open log folder; logging to stderr");
            None
        }
    }
}
