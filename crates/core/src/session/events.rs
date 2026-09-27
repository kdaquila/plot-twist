use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::plot::PlotConfig;
use crate::dataset::DatasetSummary;
use crate::errors::ErrorReport;

/// Which client triggered a change.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "lowercase")]
#[ts(export)]
pub enum Origin {
    Gui,
    Api,
}

/// Published on every session change so every client (notably the GUI) stays in sync.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(tag = "type", rename_all = "snake_case")]
#[ts(export)]
pub enum SessionEvent {
    LoadStarted {
        path: String,
        origin: Origin,
    },
    DatasetLoaded {
        dataset: DatasetSummary,
        origin: Origin,
    },
    LoadFailed {
        path: String,
        report: ErrorReport,
        origin: Origin,
    },
    PlotChanged {
        plot: PlotConfig,
        origin: Origin,
    },
}
