//! The closed error vocabulary (constitution Principle V). Codes are stable public API;
//! see `specs/001-csv-plot-explorer/contracts/error-codes.md`.

mod report;

pub use report::{ErrorReport, format_count, truncate_value};

use serde_json::json;

/// Warning code for a cell that doesn't match its column's type.
pub const BAD_CELL: &str = "BAD_CELL";

/// Every failure a user or API client can see.
#[derive(Debug, Clone, PartialEq)]
pub enum PtError {
    FileNotFound {
        path: String,
    },
    FileUnreadable {
        path: String,
        os_error: String,
    },
    EmptyFile,
    NoDataRows,
    FieldCountMismatch {
        line: u64,
        expected: usize,
        actual: usize,
    },
    InvalidEncoding {
        line: u64,
        column: String,
    },
    MalformedQuoting {
        line: u64,
    },
    OutOfMemory {
        file_size_bytes: u64,
    },
    LoadInProgress,
    UnknownDataset {
        dataset_id: String,
    },
    StaleDataset {
        dataset_id: String,
        current_dataset_id: String,
    },
    UnknownColumn {
        column: String,
    },
    ColumnNotUsableAsX {
        column: String,
    },
    ColumnNotUsableAsY {
        column: String,
    },
    XAlsoY {
        column: String,
    },
    NoYColumns,
    DuplicateYColumn {
        column: String,
    },
    NoPlot,
    InvalidView {
        reason: String,
    },
    BadRequest {
        reason: String,
    },
    Internal {
        log_id: String,
    },
}

impl PtError {
    /// Logs an unexpected fault and returns an `INTERNAL` error carrying its log id.
    pub fn internal(error: &dyn std::fmt::Display) -> Self {
        let log_id = format!(
            "{:x}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_or(0, |d| d.as_nanos())
        );
        tracing::error!(%log_id, %error, "internal error");
        Self::Internal { log_id }
    }

    /// Stable, machine-readable code.
    pub fn code(&self) -> &'static str {
        match self {
            Self::FileNotFound { .. } => "FILE_NOT_FOUND",
            Self::FileUnreadable { .. } => "FILE_UNREADABLE",
            Self::EmptyFile => "EMPTY_FILE",
            Self::NoDataRows => "NO_DATA_ROWS",
            Self::FieldCountMismatch { .. } => "FIELD_COUNT_MISMATCH",
            Self::InvalidEncoding { .. } => "INVALID_ENCODING",
            Self::MalformedQuoting { .. } => "MALFORMED_QUOTING",
            Self::OutOfMemory { .. } => "OUT_OF_MEMORY",
            Self::LoadInProgress => "LOAD_IN_PROGRESS",
            Self::UnknownDataset { .. } => "UNKNOWN_DATASET",
            Self::StaleDataset { .. } => "STALE_DATASET",
            Self::UnknownColumn { .. } => "UNKNOWN_COLUMN",
            Self::ColumnNotUsableAsX { .. } => "COLUMN_NOT_USABLE_AS_X",
            Self::ColumnNotUsableAsY { .. } => "COLUMN_NOT_USABLE_AS_Y",
            Self::XAlsoY { .. } => "X_ALSO_Y",
            Self::NoYColumns => "NO_Y_COLUMNS",
            Self::DuplicateYColumn { .. } => "DUPLICATE_Y_COLUMN",
            Self::NoPlot => "NO_PLOT",
            Self::InvalidView { .. } => "INVALID_VIEW",
            Self::BadRequest { .. } => "BAD_REQUEST",
            Self::Internal { .. } => "INTERNAL",
        }
    }

    /// What went wrong, in plain English.
    pub fn message(&self) -> String {
        match self {
            Self::FileNotFound { path } => format!("The file \"{path}\" was not found."),
            Self::FileUnreadable { path, os_error } => {
                format!("The file \"{path}\" could not be read ({os_error}).")
            }
            Self::EmptyFile => "The file contains no data.".into(),
            Self::NoDataRows => "The file has a header row but no data rows.".into(),
            Self::FieldCountMismatch {
                line,
                expected,
                actual,
            } => format!(
                "Line {} has {actual} fields, but {expected} were expected.",
                format_count(*line)
            ),
            Self::InvalidEncoding { line, column } => format!(
                "Line {}, column \"{column}\" contains text that is not valid UTF-8.",
                format_count(*line)
            ),
            Self::MalformedQuoting { line } => format!(
                "The quoted field starting on line {} is never closed.",
                format_count(*line)
            ),
            Self::OutOfMemory { .. } => "There is not enough memory to load this file.".into(),
            Self::LoadInProgress => "Another file is still loading.".into(),
            Self::UnknownDataset { dataset_id } => format!("Dataset \"{dataset_id}\" not found."),
            Self::StaleDataset { dataset_id, .. } => {
                format!("Dataset \"{dataset_id}\" is no longer the loaded dataset.")
            }
            Self::UnknownColumn { column } => format!("There is no column named \"{column}\"."),
            Self::ColumnNotUsableAsX { column } => {
                format!("Column \"{column}\" is text and can't be used for X.")
            }
            Self::ColumnNotUsableAsY { column } => {
                format!("Column \"{column}\" is not numeric and can't be used for Y.")
            }
            Self::XAlsoY { column } => {
                format!("Column \"{column}\" is the X column and can't also be a Y column.")
            }
            Self::NoYColumns => "No Y columns were chosen.".into(),
            Self::DuplicateYColumn { column } => {
                format!("Column \"{column}\" is listed more than once for Y.")
            }
            Self::NoPlot => "Nothing is plotted yet.".into(),
            Self::InvalidView { reason } => format!("The requested view is invalid: {reason}."),
            Self::BadRequest { reason } => format!("The request is invalid: {reason}."),
            Self::Internal { .. } => "Something went wrong inside plot-twist.".into(),
        }
    }

    /// Suggested next step.
    pub fn hint(&self) -> &'static str {
        match self {
            Self::FileNotFound { .. } => "Check the path, or pick the file again with File → Open.",
            Self::FileUnreadable { .. } => {
                "Close any program that has the file open and check you have permission to read it."
            }
            Self::EmptyFile => "Choose a file that contains data.",
            Self::NoDataRows => "Add at least one data row below the header.",
            Self::FieldCountMismatch { .. } => {
                "Add or remove fields on that line so it matches the header, then reopen the file."
            }
            Self::InvalidEncoding { .. } => "Re-save the file as UTF-8 (in Excel: \"CSV UTF-8\").",
            Self::MalformedQuoting { .. } => "Close or remove the unmatched quote on that line.",
            Self::OutOfMemory { .. } => {
                "Close other programs or split the file into smaller files."
            }
            Self::LoadInProgress => "Wait for the current load to finish, then try again.",
            Self::UnknownDataset { .. } | Self::StaleDataset { .. } => {
                "Get the current dataset id from GET /v1/state and retry."
            }
            Self::UnknownColumn { .. } => "Use one of the column names listed for the loaded file.",
            Self::ColumnNotUsableAsX { .. } => "Choose a numeric or date/time column for X.",
            Self::ColumnNotUsableAsY { .. } => "Choose numeric columns for Y.",
            Self::XAlsoY { .. } => "Remove the X column from the Y list.",
            Self::NoYColumns => "Choose at least one Y column.",
            Self::DuplicateYColumn { .. } => "List each Y column only once.",
            Self::NoPlot => "Set a plot with PUT /v1/plot first.",
            Self::InvalidView { .. } => {
                "Use finite ranges with min < max and sizes between 1 and 8192."
            }
            Self::BadRequest { .. } => "Check the request body against the API documentation.",
            Self::Internal { .. } => "Please report this issue and include the log id.",
        }
    }

    /// Serializable report shown by the GUI and returned by the API.
    pub fn report(&self) -> ErrorReport {
        let mut report = ErrorReport::new(self.code(), self.message(), self.hint());
        match self {
            Self::FileNotFound { path } => report.details = Some(json!({ "path": path })),
            Self::FileUnreadable { path, os_error } => {
                report.details = Some(json!({ "path": path, "os_error": os_error }));
            }
            Self::FieldCountMismatch {
                line,
                expected,
                actual,
            } => {
                report.line = Some(*line);
                report.details =
                    Some(json!({ "expected_fields": expected, "actual_fields": actual }));
            }
            Self::InvalidEncoding { line, column } => {
                report.line = Some(*line);
                report.column = Some(column.clone());
            }
            Self::MalformedQuoting { line } => report.line = Some(*line),
            Self::OutOfMemory { file_size_bytes } => {
                report.details = Some(json!({ "file_size_bytes": file_size_bytes }));
            }
            Self::UnknownDataset { dataset_id } => {
                report.details = Some(json!({ "dataset_id": dataset_id }));
            }
            Self::StaleDataset {
                dataset_id,
                current_dataset_id,
            } => {
                report.details = Some(json!({
                    "dataset_id": dataset_id,
                    "current_dataset_id": current_dataset_id,
                }));
            }
            Self::UnknownColumn { column }
            | Self::ColumnNotUsableAsX { column }
            | Self::ColumnNotUsableAsY { column }
            | Self::XAlsoY { column }
            | Self::DuplicateYColumn { column } => report.column = Some(column.clone()),
            Self::InvalidView { reason } | Self::BadRequest { reason } => {
                report.details = Some(json!({ "reason": reason }));
            }
            Self::Internal { log_id } => report.details = Some(json!({ "log_id": log_id })),
            Self::EmptyFile
            | Self::NoDataRows
            | Self::LoadInProgress
            | Self::NoYColumns
            | Self::NoPlot => {}
        }
        report
    }
}

impl std::fmt::Display for PtError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.code(), self.message())
    }
}

impl std::error::Error for PtError {}
