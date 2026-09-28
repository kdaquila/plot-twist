//! The single operation layer shared by the GUI and the local API (constitution
//! Principle I). Adapters call these methods; every change is published as a
//! [`SessionEvent`].

mod events;
mod plot;

use std::path::Path;
use std::sync::{Arc, Mutex, MutexGuard};

use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;
use ts_rs::TS;

pub use events::{ApiStatus, Origin, SessionEvent};
pub use plot::{PlotConfig, PlotStyle};

use crate::csv_import::import_file;
use crate::dataset::{BadCell, BadCellPage, Dataset, DatasetId, DatasetSummary};
use crate::errors::PtError;
use crate::settings::SettingsStore;
use crate::view::{ViewPayload, ViewRequest, reduce};

const PREVIEW_BAD_CELLS: u64 = 100;
const MAX_BAD_CELL_PAGE: u64 = 1000;

/// Locks a mutex, recovering the data if a previous holder panicked.
pub(crate) fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct LoadingInfo {
    pub path: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct SessionState {
    pub dataset: Option<DatasetSummary>,
    pub plot: Option<PlotConfig>,
    pub loading: Option<LoadingInfo>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct LoadResult {
    pub dataset: DatasetSummary,
    /// The first bad cells (at most 100); page the rest with `bad_cells`.
    pub bad_cells_preview: Vec<BadCell>,
}

#[derive(Default)]
struct State {
    dataset: Option<Arc<Dataset>>,
    plot: Option<PlotConfig>,
    loading: Option<String>,
    last_id: u64,
}

pub struct Session {
    state: Mutex<State>,
    events: broadcast::Sender<SessionEvent>,
    settings: SettingsStore,
}

impl Session {
    pub fn new(settings: SettingsStore) -> Self {
        let (events, _) = broadcast::channel(64);
        Self {
            state: Mutex::new(State::default()),
            events,
            settings,
        }
    }

    pub fn settings(&self) -> &SettingsStore {
        &self.settings
    }

    pub fn subscribe(&self) -> broadcast::Receiver<SessionEvent> {
        self.events.subscribe()
    }

    fn emit(&self, event: SessionEvent) {
        // No subscribers is fine.
        let _ = self.events.send(event);
    }

    pub fn state(&self) -> SessionState {
        let state = lock(&self.state);
        SessionState {
            dataset: state.dataset.as_ref().map(|d| d.summary()),
            plot: state.plot.clone(),
            loading: state.loading.clone().map(|path| LoadingInfo { path }),
        }
    }

    /// Loads a file, replacing the current dataset only on success (FR-004). Blocking:
    /// adapters run it on a worker thread. The session lock is not held while parsing.
    pub fn load_file(&self, path: &Path, origin: Origin) -> Result<LoadResult, PtError> {
        let path_text = path.display().to_string();
        {
            let mut state = lock(&self.state);
            if state.loading.is_some() {
                return Err(PtError::LoadInProgress);
            }
            state.loading = Some(path_text.clone());
        }
        let _loading = LoadingGuard(&self.state);
        self.emit(SessionEvent::LoadStarted {
            path: path_text.clone(),
            origin,
        });

        let imported = match import_file(path) {
            Ok(imported) => imported,
            Err(error) => {
                tracing::info!(path = %path_text, code = error.code(), "load failed");
                self.emit(SessionEvent::LoadFailed {
                    path: path_text,
                    report: error.report(),
                    origin,
                });
                return Err(error);
            }
        };
        let dataset = {
            let mut state = lock(&self.state);
            state.last_id += 1;
            let id = DatasetId::new(state.last_id);
            let dataset = Arc::new(Dataset::new(
                id,
                path,
                imported.file_size_bytes,
                imported.table,
            ));
            state.dataset = Some(Arc::clone(&dataset));
            state.plot = None;
            state.loading = None;
            dataset
        };
        self.settings.update(|s| s.record_recent(&path_text));
        let summary = dataset.summary();
        tracing::info!(path = %path_text, rows = summary.row_count, bad = summary.bad_cell_total, "loaded");
        self.emit(SessionEvent::DatasetLoaded {
            dataset: summary.clone(),
            origin,
        });
        let preview = dataset.bad_cells(0, PREVIEW_BAD_CELLS).items;
        Ok(LoadResult {
            dataset: summary,
            bad_cells_preview: preview,
        })
    }

    /// Replaces the plot; on failure the current plot is unchanged.
    pub fn set_plot(&self, config: PlotConfig, origin: Origin) -> Result<PlotConfig, PtError> {
        {
            let mut state = lock(&self.state);
            plot::validate(&config, state.dataset.as_deref())?;
            state.plot = Some(config.clone());
        }
        self.emit(SessionEvent::PlotChanged {
            plot: config.clone(),
            origin,
        });
        Ok(config)
    }

    pub fn bad_cells(
        &self,
        id: DatasetId,
        offset: u64,
        limit: u64,
    ) -> Result<BadCellPage, PtError> {
        if !(1..=MAX_BAD_CELL_PAGE).contains(&limit) {
            return Err(PtError::BadRequest {
                reason: "limit must be between 1 and 1000".into(),
            });
        }
        Ok(self.current_dataset(id)?.bad_cells(offset, limit))
    }

    /// Reduced render data for the current plot over the requested range.
    pub fn view(&self, request: &ViewRequest) -> Result<ViewPayload, PtError> {
        let (dataset, plot) = {
            let state = lock(&self.state);
            let plot = state.plot.clone().ok_or(PtError::NoPlot)?;
            let dataset = state.dataset.clone().ok_or(PtError::NoPlot)?;
            (dataset, plot)
        };
        if request.dataset_id != dataset.id {
            return Err(PtError::StaleDataset {
                dataset_id: request.dataset_id.to_string(),
                current_dataset_id: dataset.id.to_string(),
            });
        }
        reduce(&dataset, &plot, request)
    }

    fn current_dataset(&self, id: DatasetId) -> Result<Arc<Dataset>, PtError> {
        lock(&self.state)
            .dataset
            .clone()
            .filter(|d| d.id == id)
            .ok_or_else(|| PtError::UnknownDataset {
                dataset_id: id.to_string(),
            })
    }
}

/// Clears `loading` however the load ends.
struct LoadingGuard<'a>(&'a Mutex<State>);

impl Drop for LoadingGuard<'_> {
    fn drop(&mut self) {
        lock(self.0).loading = None;
    }
}
