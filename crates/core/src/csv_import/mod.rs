//! Reads a CSV file into typed columns, recording bad cells and failing on structural
//! problems (spec Edge Cases, FR-002, FR-011a).

mod cells;
mod columns;
mod header;
mod sniff;

use std::fs::File;
use std::io::{BufReader, Read, Seek, SeekFrom};
use std::path::Path;

use csv::ByteRecord;

use crate::dataset::{BadCellStore, Delimiter, Table};
use crate::errors::PtError;
use columns::{ColumnBuilder, OutOfMemory};

pub use cells::{Cell, classify, parse_datetime};

const BOM: &[u8] = b"\xEF\xBB\xBF";
const SAMPLE_BYTES: usize = 64 * 1024;

/// A parsed file and its size on disk.
pub struct ImportedFile {
    pub file_size_bytes: u64,
    pub table: Table,
}

/// Reads and parses the CSV at `path`.
pub fn import_file(path: &Path) -> Result<ImportedFile, PtError> {
    let display = path.display().to_string();
    let metadata = std::fs::metadata(path).map_err(|e| io_error(&display, &e))?;
    if metadata.is_dir() {
        return Err(PtError::FileUnreadable {
            path: display,
            os_error: "it is a folder".into(),
        });
    }
    let file_size_bytes = metadata.len();
    let mut file = File::open(path).map_err(|e| io_error(&display, &e))?;

    let mut sample = Vec::with_capacity(SAMPLE_BYTES);
    (&mut file)
        .take(SAMPLE_BYTES as u64)
        .read_to_end(&mut sample)
        .map_err(|e| io_error(&display, &e))?;
    let skip = if sample.starts_with(BOM) {
        BOM.len()
    } else {
        0
    };
    let body = sample.get(skip..).unwrap_or_default();
    let delimiter = sniff::sniff(body, sample.len() < SAMPLE_BYTES);
    let estimated_rows = estimate_rows(body, file_size_bytes);
    file.seek(SeekFrom::Start(skip as u64))
        .map_err(|e| io_error(&display, &e))?;

    let reader = csv::ReaderBuilder::new()
        .has_headers(false)
        .flexible(true)
        .delimiter(delimiter.unwrap_or(Delimiter::Comma).byte())
        .from_reader(BufReader::with_capacity(1 << 20, file));
    let parser = Parser::new(
        delimiter.unwrap_or(Delimiter::Comma),
        estimated_rows,
        file_size_bytes,
    );
    let table = parser.run(reader, &display)?;
    Ok(ImportedFile {
        file_size_bytes,
        table,
    })
}

fn io_error(path: &str, error: &std::io::Error) -> PtError {
    if error.kind() == std::io::ErrorKind::NotFound {
        PtError::FileNotFound {
            path: path.to_owned(),
        }
    } else {
        PtError::FileUnreadable {
            path: path.to_owned(),
            os_error: error.to_string(),
        }
    }
}

fn estimate_rows(sample: &[u8], file_size: u64) -> usize {
    let lines = sample.iter().filter(|&&b| b == b'\n').count().max(1);
    let per_line = (sample.len() / lines).max(1) as u64;
    usize::try_from(file_size / per_line + 16)
        .unwrap_or(usize::MAX)
        .min(1 << 28)
}

struct Parser {
    delimiter: Delimiter,
    estimated_rows: usize,
    file_size_bytes: u64,
    names: Vec<String>,
    builders: Vec<ColumnBuilder>,
    has_header: bool,
    row_count: u64,
    bad_cells: BadCellStore,
}

impl Parser {
    fn new(delimiter: Delimiter, estimated_rows: usize, file_size_bytes: u64) -> Self {
        Self {
            delimiter,
            estimated_rows,
            file_size_bytes,
            names: Vec::new(),
            builders: Vec::new(),
            has_header: false,
            row_count: 0,
            bad_cells: BadCellStore::default(),
        }
    }

    fn run<R: Read>(mut self, mut reader: csv::Reader<R>, path: &str) -> Result<Table, PtError> {
        let mut record = ByteRecord::new();
        loop {
            match reader.read_byte_record(&mut record) {
                Ok(true) => {}
                Ok(false) => break,
                Err(e) => return Err(csv_error(&e, path)),
            }
            let line = record.position().map_or(0, csv::Position::line);
            if is_blank(&record) {
                continue;
            }
            if self.builders.is_empty() {
                if self.start(&record, line)? {
                    continue;
                }
            } else {
                self.check_width(&record, line)?;
            }
            self.push_row(&record, line)?;
        }
        self.finish()
    }

    /// Handles the first non-blank record. Returns `true` if it was a header.
    fn start(&mut self, record: &ByteRecord, line: u64) -> Result<bool, PtError> {
        let fields = record
            .iter()
            .enumerate()
            .map(|(i, field)| {
                std::str::from_utf8(field).map_err(|_| PtError::InvalidEncoding {
                    line,
                    column: format!("Column {}", i + 1),
                })
            })
            .collect::<Result<Vec<&str>, PtError>>()?;
        self.has_header = !header::looks_like_data(&fields);
        let header_fields = self.has_header.then_some(fields.as_slice());
        self.names = header::column_names(header_fields, fields.len());
        for index in 0..fields.len() {
            let index = u32::try_from(index).unwrap_or(u32::MAX);
            let builder = ColumnBuilder::new(index, self.estimated_rows)
                .map_err(|OutOfMemory| self.out_of_memory())?;
            self.builders.push(builder);
        }
        Ok(self.has_header)
    }

    fn check_width(&self, record: &ByteRecord, line: u64) -> Result<(), PtError> {
        if record.len() == self.builders.len() {
            return Ok(());
        }
        if record.iter().any(|field| field.contains(&b'\n')) {
            return Err(PtError::MalformedQuoting { line });
        }
        Err(PtError::FieldCountMismatch {
            line,
            expected: self.builders.len(),
            actual: record.len(),
        })
    }

    fn push_row(&mut self, record: &ByteRecord, line: u64) -> Result<(), PtError> {
        for (i, (field, builder)) in record.iter().zip(self.builders.iter_mut()).enumerate() {
            let text = std::str::from_utf8(field).map_err(|_| PtError::InvalidEncoding {
                line,
                column: self.names.get(i).cloned().unwrap_or_default(),
            })?;
            if builder.push(text, line, &mut self.bad_cells).is_err() {
                return Err(PtError::OutOfMemory {
                    file_size_bytes: self.file_size_bytes,
                });
            }
        }
        self.row_count += 1;
        Ok(())
    }

    fn out_of_memory(&self) -> PtError {
        PtError::OutOfMemory {
            file_size_bytes: self.file_size_bytes,
        }
    }

    fn finish(self) -> Result<Table, PtError> {
        if self.builders.is_empty() {
            return Err(PtError::EmptyFile);
        }
        if self.row_count == 0 {
            return Err(PtError::NoDataRows);
        }
        let columns = self
            .builders
            .into_iter()
            .zip(self.names)
            .map(|(builder, name)| builder.finish(name))
            .collect();
        Ok(Table {
            has_header: self.has_header,
            delimiter: self.delimiter,
            row_count: self.row_count,
            columns,
            bad_cells: self.bad_cells,
        })
    }
}

/// A record holding a single whitespace-only field is a blank line.
fn is_blank(record: &ByteRecord) -> bool {
    record.len() == 1
        && record
            .get(0)
            .is_some_and(|f| f.iter().all(u8::is_ascii_whitespace))
}

fn csv_error(error: &csv::Error, path: &str) -> PtError {
    let line = error.position().map_or(0, csv::Position::line);
    match error.kind() {
        csv::ErrorKind::Io(io) => io_error(path, io),
        _ => PtError::MalformedQuoting { line },
    }
}
