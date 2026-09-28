//! Starts the local API and reports its address (or failure) to the GUI (FR-005c).

use std::sync::{Arc, Mutex};

use plot_twist_api::ApiHandle;
use plot_twist_core::session::{ApiStatus, Session};
use tauri::{AppHandle, Emitter, Manager};

use super::{paths, ready};

pub const API_STATUS: &str = "api-status";

#[derive(Default)]
pub struct ApiState {
    status: Mutex<Option<ApiStatus>>,
    handle: Mutex<Option<ApiHandle>>,
}

impl ApiState {
    pub fn status(&self) -> Option<ApiStatus> {
        self.status.lock().ok().and_then(|s| s.clone())
    }

    /// Stops the server and removes the discovery file.
    pub fn stop(&self) {
        if let Ok(mut handle) = self.handle.lock()
            && let Some(mut handle) = handle.take()
        {
            handle.stop();
        }
    }
}

pub fn start(app: &AppHandle, session: Arc<Session>) {
    app.manage(ApiState::default());
    let port = session.settings().get().api_port;
    let app = app.clone();
    tauri::async_runtime::spawn(async move {
        let status = match plot_twist_api::start(session, port, paths::api_discovery_file()).await {
            Ok(handle) => {
                let status = ApiStatus::Running {
                    base_url: handle.base_url().to_owned(),
                };
                if let Ok(mut slot) = app.state::<ApiState>().handle.lock() {
                    *slot = Some(handle);
                }
                status
            }
            Err(error) => ApiStatus::Failed {
                error: error.report(),
            },
        };
        if let Ok(mut slot) = app.state::<ApiState>().status.lock() {
            *slot = Some(status.clone());
        }
        if let Err(error) = app.emit(API_STATUS, &status) {
            tracing::error!(%error, "could not send API status to the window");
        }
        ready::mark(&app, ready::API);
    });
}
