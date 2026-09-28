//! App startup: logging, single instance, session, event forwarding, local API.

mod api;
mod dev_server;
mod logging;
mod paths;
pub mod ready;

pub use api::ApiState;

use std::sync::Arc;

use plot_twist_core::session::Session;
use plot_twist_core::settings::SettingsStore;
use tauri::{App, Manager, RunEvent};

use crate::{commands, events};

pub fn run() {
    let _log_guard = logging::init();
    let built = tauri::Builder::default()
        // Must be first: a second launch focuses the existing window instead.
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            focus_main(app)
        }))
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            setup(app);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::data::load_file,
            commands::data::get_state,
            commands::data::set_plot,
            commands::data::get_bad_cells,
            commands::data::get_view,
            commands::settings::get_settings,
            commands::settings::set_theme,
            commands::settings::remove_recent_file,
            commands::app::get_api_status,
            commands::app::frontend_ready,
        ])
        .build(tauri::generate_context!());
    match built {
        Ok(app) => app.run(|app, event| {
            if let RunEvent::Exit = event
                && let Some(api) = app.try_state::<ApiState>()
            {
                api.stop();
            }
        }),
        Err(error) => tracing::error!(%error, "could not start plot-twist"),
    }
}

fn setup(app: &mut App) {
    tracing::info!(version = env!("CARGO_PKG_VERSION"), "starting");
    dev_server::check(app);
    let session = Arc::new(Session::new(SettingsStore::open(paths::settings_file())));
    events::forward_session_events(app.handle().clone(), &session);
    api::start(app.handle(), Arc::clone(&session));
    app.manage(session);
}

fn focus_main(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.unminimize();
        let _ = window.set_focus();
    }
}
