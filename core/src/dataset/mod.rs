//! A loaded file: its columns, values, and bad cells.

mod bad_cells;
mod column;
mod ids;

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use ts_rs::TS;

pub use bad_cells::{BadCell, BadCellPage, BadCellStore};
pub use column::{Column, ColumnInfo, ColumnKind};
pub use ids::{ColumnIndex, DatasetId, LineNumber};

/// Detected field delimiter.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "lowercase")]
#[ts(export)]
pub enum Delimiter {
    Comma,
    Semicolon,
    Tab,
}

impl Delimiter {
    pub fn byte(self) -> u8 {
        match self {
            Self::Comma => b',',
            Self::Semicolon => b';',
            Self::Tab => b'\t',
        }
    }
}

/// The parsed contents of a file, before it becomes a dataset.
#[derive(Debug)]
pub struct Table {
    pub has_header: bool,
    pub delimiter: Delimiter,
    pub row_count: u64,
    pub columns: Vec<Column>,
    pub bad_cells: BadCellStore,
}

/// One successfully loaded file. Never mutated after creation.
#[derive(Debug)]
pub struct Dataset {
    pub id: DatasetId,
    pub path: PathBuf,
    pub file_name: String,
    pub file_size_bytes: u64,
    pub table: Table,
}

impl Dataset {
    pub fn new(id: DatasetId, path: &Path, file_size_bytes: u64, table: Table) -> Self {
        let file_name = path
            .file_name()
            .map_or_else(|| path.to_string_lossy(), |n| n.to_string_lossy())
            .into_owned();
        Self {
            id,
            path: path.to_path_buf(),
            file_name,
            file_size_bytes,
            table,
        }
    }

    pub fn column_by_name(&self, name: &str) -> Option<&Column> {
        self.table.columns.iter().find(|c| c.name == name)
    }

    pub fn column_names(&self) -> Vec<&str> {
        self.table.columns.iter().map(|c| c.name.as_str()).collect()
    }

    pub fn bad_cells(&self, offset: u64, limit: u64) -> BadCellPage {
        self.table
            .bad_cells
            .page(offset, limit, &self.column_names())
    }

    pub fn summary(&self) -> DatasetSummary {
        DatasetSummary {
            id: self.id,
            path: self.path.to_string_lossy().into_owned(),
            file_name: self.file_name.clone(),
            file_size_bytes: self.file_size_bytes,
            row_count: self.table.row_count,
            has_header: self.table.has_header,
            delimiter: self.table.delimiter,
            columns: self.table.columns.iter().map(Column::info).collect(),
            bad_cell_total: self.table.bad_cells.total(),
        }
    }
}

/// Serializable description of a dataset (no values, no bad-cell entries).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct DatasetSummary {
    pub id: DatasetId,
    pub path: String,
    pub file_name: String,
    #[ts(type = "number")]
    pub file_size_bytes: u64,
    #[ts(type = "number")]
    pub row_count: u64,
    pub has_header: bool,
    pub delimiter: Delimiter,
    pub columns: Vec<ColumnInfo>,
    #[ts(type = "number")]
    pub bad_cell_total: u64,
}
