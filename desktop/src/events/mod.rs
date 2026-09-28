//! Forwards every session change to the WebView as `session-changed` (FR-015).

use std::sync::Arc;

use plot_twist_core::session::Session;
use tauri::{AppHandle, Emitter};
use tokio::sync::broadcast::error::RecvError;

pub const SESSION_CHANGED: &str = "session-changed";

pub fn forward_session_events(app: AppHandle, session: &Arc<Session>) {
    let mut events = session.subscribe();
    tauri::async_runtime::spawn(async move {
        loop {
            match events.recv().await {
                Ok(event) => {
                    if let Err(error) = app.emit(SESSION_CHANGED, &event) {
                        tracing::warn!(%error, "could not forward session event");
                    }
                }
                Err(RecvError::Lagged(skipped)) => {
                    tracing::warn!(skipped, "GUI fell behind on session events");
                }
                Err(RecvError::Closed) => break,
            }
        }
    });
}
