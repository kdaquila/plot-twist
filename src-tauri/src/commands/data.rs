use std::path::PathBuf;
use std::sync::Arc;

use plot_twist_core::dataset::{BadCellPage, DatasetId};
use plot_twist_core::errors::ErrorReport;
use plot_twist_core::session::{LoadResult, Origin, PlotConfig, Session, SessionState};
use plot_twist_core::view::{ViewRequest, encode};
use tauri::State;
use tauri::ipc::Response;

use super::internal;

type Shared<'a> = State<'a, Arc<Session>>;

/// Runs a blocking session operation off the main thread.
async fn blocking<T: Send + 'static>(
    session: &Shared<'_>,
    op: impl FnOnce(&Session) -> Result<T, plot_twist_core::errors::PtError> + Send + 'static,
) -> Result<T, ErrorReport> {
    let session = Arc::clone(session);
    tauri::async_runtime::spawn_blocking(move || op(&session))
        .await
        .map_err(|e| internal(&e))?
        .map_err(|e| e.report())
}

#[tauri::command]
pub async fn load_file(session: Shared<'_>, path: String) -> Result<LoadResult, ErrorReport> {
    blocking(&session, move |s| {
        s.load_file(&PathBuf::from(path), Origin::Gui)
    })
    .await
}

#[tauri::command]
pub fn get_state(session: Shared<'_>) -> SessionState {
    session.state()
}

#[tauri::command]
pub fn set_plot(session: Shared<'_>, config: PlotConfig) -> Result<PlotConfig, ErrorReport> {
    session
        .set_plot(config, Origin::Gui)
        .map_err(|e| e.report())
}

#[tauri::command]
pub fn get_bad_cells(
    session: Shared<'_>,
    dataset_id: DatasetId,
    offset: u64,
    limit: u64,
) -> Result<BadCellPage, ErrorReport> {
    session
        .bad_cells(dataset_id, offset, limit)
        .map_err(|e| e.report())
}

/// Binary payload (contracts/tauri-commands.md) to avoid JSON-encoding float arrays.
#[tauri::command]
pub async fn get_view(session: Shared<'_>, request: ViewRequest) -> Result<Response, ErrorReport> {
    let payload = blocking(&session, move |s| s.view(&request)).await?;
    Ok(Response::new(encode(&payload)))
}
