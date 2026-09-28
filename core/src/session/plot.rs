use std::collections::HashSet;

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::dataset::{Column, Dataset, DatasetId};
use crate::errors::PtError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "lowercase")]
#[ts(export)]
pub enum PlotStyle {
    Line,
    Scatter,
}

/// What is plotted. Columns are referenced by their unique display names.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct PlotConfig {
    pub dataset_id: DatasetId,
    pub x: String,
    pub y: Vec<String>,
    pub style: PlotStyle,
}

/// Checks `config` against the loaded dataset, in the order given in data-model.md.
pub fn validate(config: &PlotConfig, dataset: Option<&Dataset>) -> Result<(), PtError> {
    let Some(dataset) = dataset else {
        return Err(PtError::UnknownDataset {
            dataset_id: config.dataset_id.to_string(),
        });
    };
    if config.dataset_id != dataset.id {
        return Err(if config.dataset_id < dataset.id {
            PtError::StaleDataset {
                dataset_id: config.dataset_id.to_string(),
                current_dataset_id: dataset.id.to_string(),
            }
        } else {
            PtError::UnknownDataset {
                dataset_id: config.dataset_id.to_string(),
            }
        });
    }
    let x = column(dataset, &config.x)?;
    if !x.usable_as_x() {
        return Err(PtError::ColumnNotUsableAsX {
            column: config.x.clone(),
        });
    }
    if config.y.is_empty() {
        return Err(PtError::NoYColumns);
    }
    let mut seen = HashSet::new();
    for name in &config.y {
        if !seen.insert(name.as_str()) {
            return Err(PtError::DuplicateYColumn {
                column: name.clone(),
            });
        }
        if !column(dataset, name)?.usable_as_y() {
            return Err(PtError::ColumnNotUsableAsY {
                column: name.clone(),
            });
        }
        if *name == config.x {
            return Err(PtError::XAlsoY {
                column: name.clone(),
            });
        }
    }
    Ok(())
}

fn column<'a>(dataset: &'a Dataset, name: &str) -> Result<&'a Column, PtError> {
    dataset
        .column_by_name(name)
        .ok_or_else(|| PtError::UnknownColumn {
            column: name.to_owned(),
        })
}
