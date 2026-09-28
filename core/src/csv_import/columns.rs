//! Per-column accumulation of typed values while reading.

use super::cells::{Cell, classify, is_missing, parse_datetime, parse_number};
use crate::dataset::{BadCellStore, Column, ColumnIndex, ColumnKind};

/// Memory could not be reserved.
pub struct OutOfMemory;

/// Collects one column's values; its kind is set by the first typed value.
pub struct ColumnBuilder {
    index: u32,
    kind: Option<ColumnKind>,
    has_offset: bool,
    values: Vec<f64>,
    missing: u64,
    bad: u64,
}

impl ColumnBuilder {
    pub fn new(index: u32, estimated_rows: usize) -> Result<Self, OutOfMemory> {
        let mut values = Vec::new();
        values
            .try_reserve_exact(estimated_rows)
            .map_err(|_| OutOfMemory)?;
        Ok(Self {
            index,
            kind: None,
            has_offset: false,
            values,
            missing: 0,
            bad: 0,
        })
    }

    /// Adds one raw cell from data row on `line`.
    pub fn push(
        &mut self,
        raw: &str,
        line: u64,
        bad_cells: &mut BadCellStore,
    ) -> Result<(), OutOfMemory> {
        let text = raw.trim();
        match self.kind {
            Some(ColumnKind::Text) => {
                if is_missing(text) {
                    self.missing += 1;
                }
                Ok(())
            }
            Some(ColumnKind::Numeric) => match parse_number(text) {
                Some(v) if v.is_finite() => self.value(v),
                Some(_) => self.missing(),
                None if is_missing(text) => self.missing(),
                None => self.bad(text, line, bad_cells),
            },
            Some(ColumnKind::DateTime) => match parse_datetime(text) {
                Some((seconds, offset)) if offset == self.has_offset => self.value(seconds),
                _ if is_missing(text) => self.missing(),
                _ => self.bad(text, line, bad_cells),
            },
            None => self.push_untyped(text),
        }
    }

    fn push_untyped(&mut self, text: &str) -> Result<(), OutOfMemory> {
        match classify(text) {
            Cell::Missing => self.missing(),
            Cell::Number(v) => {
                self.kind = Some(ColumnKind::Numeric);
                self.value(v)
            }
            Cell::DateTime {
                seconds,
                has_offset,
            } => {
                self.kind = Some(ColumnKind::DateTime);
                self.has_offset = has_offset;
                self.value(seconds)
            }
            Cell::Other => {
                self.kind = Some(ColumnKind::Text);
                self.values = Vec::new();
                Ok(())
            }
        }
    }

    fn value(&mut self, v: f64) -> Result<(), OutOfMemory> {
        if self.values.len() == self.values.capacity() {
            let extra = (self.values.len() / 2).max(1024);
            self.values.try_reserve(extra).map_err(|_| OutOfMemory)?;
        }
        self.values.push(v);
        Ok(())
    }

    fn missing(&mut self) -> Result<(), OutOfMemory> {
        self.missing += 1;
        self.value(f64::NAN)
    }

    fn bad(
        &mut self,
        text: &str,
        line: u64,
        bad_cells: &mut BadCellStore,
    ) -> Result<(), OutOfMemory> {
        self.bad += 1;
        if !bad_cells.push(line, self.index, text) {
            return Err(OutOfMemory);
        }
        self.missing()
    }

    pub fn finish(self, name: String) -> Column {
        let kind = self.kind.unwrap_or(ColumnKind::Numeric);
        let values = (kind != ColumnKind::Text).then(|| {
            let mut values = self.values;
            values.shrink_to_fit();
            values
        });
        let (min, max) = values.as_deref().map_or((None, None), extent);
        let mut column = Column::new(ColumnIndex(self.index), name, kind, values);
        column.min = min;
        column.max = max;
        column.missing_count = self.missing;
        column.bad_cell_count = self.bad;
        column
    }
}

fn extent(values: &[f64]) -> (Option<f64>, Option<f64>) {
    values
        .iter()
        .filter(|v| !v.is_nan())
        .fold((None, None), |(lo, hi), &v| {
            (
                Some(lo.map_or(v, |l: f64| l.min(v))),
                Some(hi.map_or(v, |h: f64| h.max(v))),
            )
        })
}
