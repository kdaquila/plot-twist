use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::ids::{ColumnIndex, LineNumber};
use crate::errors::truncate_value;

/// A cell that could not be read as its column's type; stored as missing (NaN).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct BadCell {
    pub line: LineNumber,
    pub column: ColumnIndex,
    pub column_name: String,
    pub text: String,
}

/// A page of bad cells in file order.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct BadCellPage {
    #[ts(type = "number")]
    pub total: u64,
    #[ts(type = "number")]
    pub offset: u64,
    pub items: Vec<BadCell>,
}

#[derive(Debug, Clone, Copy)]
struct Entry {
    line: u64,
    column: u32,
    start: usize,
    len: u16,
}

/// Compact storage for every bad cell of a dataset: fixed-size entries plus one shared text
/// buffer, so files with millions of bad cells stay within the memory budget.
#[derive(Debug, Default)]
pub struct BadCellStore {
    entries: Vec<Entry>,
    text: String,
}

impl BadCellStore {
    /// Records a bad cell. Returns `false` if memory could not be reserved.
    pub fn push(&mut self, line: u64, column: u32, text: &str) -> bool {
        let stored = truncate_value(text);
        if self.entries.try_reserve(1).is_err() || self.text.try_reserve(stored.len()).is_err() {
            return false;
        }
        let start = self.text.len();
        self.text.push_str(&stored);
        let len = u16::try_from(stored.len()).unwrap_or(u16::MAX);
        self.entries.push(Entry {
            line,
            column,
            start,
            len,
        });
        true
    }

    pub fn total(&self) -> u64 {
        self.entries.len() as u64
    }

    /// Returns up to `limit` bad cells starting at `offset`, naming columns from `names`.
    pub fn page(&self, offset: u64, limit: u64, names: &[&str]) -> BadCellPage {
        let start = usize::try_from(offset).unwrap_or(usize::MAX);
        let take = usize::try_from(limit).unwrap_or(usize::MAX);
        let items = self
            .entries
            .iter()
            .skip(start)
            .take(take)
            .map(|entry| BadCell {
                line: LineNumber(entry.line),
                column: ColumnIndex(entry.column),
                column_name: names.get(entry.column as usize).map_or_else(
                    || format!("Column {}", entry.column + 1),
                    |n| (*n).to_owned(),
                ),
                text: self
                    .text
                    .get(entry.start..entry.start + usize::from(entry.len))
                    .unwrap_or_default()
                    .to_owned(),
            })
            .collect();
        BadCellPage {
            total: self.total(),
            offset,
            items,
        }
    }
}
