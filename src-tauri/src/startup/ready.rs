//! Startup readiness for the SC-003 check: with `PLOT_TWIST_EXIT_WHEN_READY=1` the app exits
//! as soon as the window has rendered and the local API is serving.

use std::sync::atomic::{AtomicU8, Ordering};

use tauri::AppHandle;

pub const WINDOW: u8 = 1;
pub const API: u8 = 2;

static READY: AtomicU8 = AtomicU8::new(0);

pub fn mark(app: &AppHandle, part: u8) {
    let now = READY.fetch_or(part, Ordering::SeqCst) | part;
    if now == WINDOW | API {
        tracing::info!("ready");
        if std::env::var_os("PLOT_TWIST_EXIT_WHEN_READY").is_some_and(|v| v == "1") {
            app.exit(0);
        }
    }
}
