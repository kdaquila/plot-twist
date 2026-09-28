//! App status (not data operations, so not part of the local API).

use plot_twist_core::session::ApiStatus;
use tauri::{AppHandle, State};

use crate::startup::{ApiState, ready};

/// `None` until the API has started or failed; `api-status` announces it too.
#[tauri::command]
pub fn get_api_status(api: State<'_, ApiState>) -> Option<ApiStatus> {
    api.status()
}

/// Called by the window after its first render.
#[tauri::command]
pub fn frontend_ready(app: AppHandle) {
    ready::mark(&app, ready::WINDOW);
}
