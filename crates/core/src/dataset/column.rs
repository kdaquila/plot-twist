use std::sync::OnceLock;

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::ids::ColumnIndex;

/// A column's data type, decided by its first typed value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "lowercase")]
#[ts(export)]
pub enum ColumnKind {
    Numeric,
    #[serde(rename = "datetime")]
    DateTime,
    Text,
}

/// One column of a loaded dataset.
#[derive(Debug)]
pub struct Column {
    pub index: ColumnIndex,
    pub name: String,
    pub kind: ColumnKind,
    pub missing_count: u64,
    pub bad_cell_count: u64,
    /// Smallest and largest non-missing values (numeric and date/time columns).
    pub min: Option<f64>,
    pub max: Option<f64>,
    /// Numeric and date/time columns only; NaN = missing. Date/time values are seconds
    /// since 1970-01-01 on a naive timeline.
    pub values: Option<Vec<f64>>,
    monotonic: OnceLock<bool>,
}

impl Column {
    pub fn new(
        index: ColumnIndex,
        name: String,
        kind: ColumnKind,
        values: Option<Vec<f64>>,
    ) -> Self {
        Self {
            index,
            name,
            kind,
            missing_count: 0,
            bad_cell_count: 0,
            min: None,
            max: None,
            values,
            monotonic: OnceLock::new(),
        }
    }

    pub fn usable_as_x(&self) -> bool {
        matches!(self.kind, ColumnKind::Numeric | ColumnKind::DateTime)
    }

    pub fn usable_as_y(&self) -> bool {
        self.kind == ColumnKind::Numeric
    }

    /// Values of a numeric or date/time column (empty for text columns).
    pub fn values(&self) -> &[f64] {
        self.values.as_deref().unwrap_or(&[])
    }

    /// Whether the non-missing values never decrease (computed once).
    pub fn is_monotonic(&self) -> bool {
        *self.monotonic.get_or_init(|| {
            let mut previous = f64::NEG_INFINITY;
            for &value in self.values() {
                if value.is_nan() {
                    continue;
                }
                if value < previous {
                    return false;
                }
                previous = value;
            }
            true
        })
    }

    pub fn info(&self) -> ColumnInfo {
        ColumnInfo {
            index: self.index,
            name: self.name.clone(),
            kind: self.kind,
            missing_count: self.missing_count,
            bad_cell_count: self.bad_cell_count,
            usable_as_x: self.usable_as_x(),
            usable_as_y: self.usable_as_y(),
            min: self.min,
            max: self.max,
        }
    }
}

/// Serializable description of a column (no values).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ColumnInfo {
    pub index: ColumnIndex,
    pub name: String,
    pub kind: ColumnKind,
    #[ts(type = "number")]
    pub missing_count: u64,
    #[ts(type = "number")]
    pub bad_cell_count: u64,
    pub usable_as_x: bool,
    pub usable_as_y: bool,
    /// Smallest non-missing value; `null` for text or all-missing columns.
    pub min: Option<f64>,
    pub max: Option<f64>,
}
