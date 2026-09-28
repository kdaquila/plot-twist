//! Acceptance: malformed files produce precise errors or bad-cell warnings, and a failed
//! load leaves the session unchanged (User Story 2, FR-011, FR-011a, FR-013).

#![allow(clippy::unwrap_used, clippy::panic)] // test helpers may fail loudly

use std::path::PathBuf;

use plot_twist_core::csv_import::import_file;
use plot_twist_core::errors::PtError;
use plot_twist_core::session::{Origin, Session};
use plot_twist_core::settings::SettingsStore;

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/csv")
        .join(name)
}

fn error(name: &str) -> PtError {
    match import_file(&fixture(name)) {
        Ok(_) => panic!("{name} should fail"),
        Err(e) => e,
    }
}

#[test]
fn bad_cells_load_as_missing_and_are_all_listed() {
    let table = import_file(&fixture("bad_cells.csv")).unwrap().table;
    let temp = table.columns.iter().find(|c| c.name == "temp").unwrap();
    assert_eq!(temp.bad_cell_count, 3);
    assert!(temp.values()[1202].is_nan());
    let page = table.bad_cells.page(0, 100, &["time", "temp", "pressure"]);
    assert_eq!(page.total, 3);
    let lines: Vec<u64> = page.items.iter().map(|c| c.line.0).collect();
    assert_eq!(lines, [50, 60, 1204]);
    assert_eq!(page.items[0].text, "#DIV/0!");
    assert_eq!(
        page.items[1].text.chars().count(),
        65,
        "long values are truncated with …"
    );
    assert_eq!(page.items[2].column_name, "temp");
    assert_eq!(page.items[2].text, "abc");
    assert_eq!(table.bad_cells.page(2, 1, &[]).items.len(), 1);
}

#[test]
fn field_count_mismatch_names_line_and_counts() {
    assert_eq!(
        error("ragged.csv"),
        PtError::FieldCountMismatch {
            line: 3,
            expected: 3,
            actual: 2
        }
    );
    let report = error("ragged.csv").report();
    assert_eq!(report.code, "FIELD_COUNT_MISMATCH");
    assert_eq!(report.line, Some(3));
    assert!(report.message.contains("Line 3"));
    assert!(!report.hint.is_empty());
    assert_eq!(
        error("ragged_no_header.csv"),
        PtError::FieldCountMismatch {
            line: 3,
            expected: 3,
            actual: 2
        }
    );
}

#[test]
fn empty_files_and_header_only() {
    assert_eq!(error("empty.csv"), PtError::EmptyFile);
    assert_eq!(error("whitespace_only.csv"), PtError::EmptyFile);
    assert_eq!(error("bom_only.csv"), PtError::EmptyFile);
    assert_eq!(error("header_only.csv"), PtError::NoDataRows);
}

#[test]
fn invalid_encoding_names_line_and_column() {
    assert_eq!(
        error("latin1.csv"),
        PtError::InvalidEncoding {
            line: 3,
            column: "name".into()
        }
    );
}

#[test]
fn unterminated_quote_is_reported() {
    assert_eq!(
        error("unterminated_quote.csv"),
        PtError::MalformedQuoting { line: 3 }
    );
}

#[test]
fn missing_file_and_folder() {
    assert!(matches!(
        error("does_not_exist.csv"),
        PtError::FileNotFound { .. }
    ));
    assert!(matches!(error(""), PtError::FileUnreadable { .. }));
}

#[test]
fn decimal_commas_and_mixed_offsets_become_bad_cells() {
    let table = import_file(&fixture("decimal_comma.csv")).unwrap().table;
    assert_eq!(table.bad_cells.total(), 4);
    let mixed = import_file(&fixture("mixed_offsets.csv")).unwrap().table;
    assert_eq!(mixed.bad_cells.total(), 1);
    assert_eq!(mixed.bad_cells.page(0, 1, &[]).items[0].line.0, 3);
}

#[test]
fn failed_load_keeps_previous_dataset_and_plot() {
    let session = Session::new(SettingsStore::in_memory());
    let loaded = session
        .load_file(&fixture("sensors.csv"), Origin::Gui)
        .unwrap();
    let plot = plot_twist_core::session::PlotConfig {
        dataset_id: loaded.dataset.id,
        x: "time".into(),
        y: vec!["temp".into()],
        style: plot_twist_core::session::PlotStyle::Line,
    };
    session.set_plot(plot.clone(), Origin::Gui).unwrap();
    assert!(
        session
            .load_file(&fixture("ragged.csv"), Origin::Gui)
            .is_err()
    );
    let state = session.state();
    assert_eq!(state.dataset.map(|d| d.id), Some(loaded.dataset.id));
    assert_eq!(state.plot, Some(plot));
    assert!(state.loading.is_none());
}
