//! App preferences (not data operations, so not part of the local API).

use std::sync::Arc;

use plot_twist_core::session::Session;
use plot_twist_core::settings::{Settings, Theme};
use tauri::State;

#[tauri::command]
pub fn get_settings(session: State<'_, Arc<Session>>) -> Settings {
    session.settings().get()
}

#[tauri::command]
pub fn set_theme(session: State<'_, Arc<Session>>, theme: Theme) -> Settings {
    session.settings().update(|s| s.theme = theme)
}

#[tauri::command]
pub fn remove_recent_file(session: State<'_, Arc<Session>>, path: String) -> Settings {
    session.settings().update(|s| s.remove_recent(&path))
}
