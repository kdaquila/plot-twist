//! Tauri command adapters. Each is a thin wrapper over the same `Session` operation the
//! local API uses (contracts/tauri-commands.md).

// `#[tauri::command]` on async fns expands to code containing `unreachable!`, and commands
// return the full `ErrorReport` so the GUI shows exactly what the API returns.
#![allow(clippy::unreachable, clippy::result_large_err)]

pub mod data;
pub mod settings;

use plot_twist_core::errors::{ErrorReport, PtError};

/// Logs an unexpected fault and returns an `INTERNAL` report carrying its log id.
pub fn internal(error: &dyn std::fmt::Display) -> ErrorReport {
    let log_id = format!(
        "{:x}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |d| d.as_nanos())
    );
    tracing::error!(%log_id, %error, "internal error");
    PtError::Internal { log_id }.report()
}
