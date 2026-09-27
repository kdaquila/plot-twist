//! Acceptance: valid files load per the spec's parsing rules (FR-002, Edge Cases).

#![allow(clippy::unwrap_used, clippy::panic)] // test helpers may fail loudly

use std::path::PathBuf;

use plot_twist_core::csv_import::import_file;
use plot_twist_core::dataset::{Column, ColumnKind, Delimiter, Table};

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/csv")
        .join(name)
}

fn load(name: &str) -> Table {
    import_file(&fixture(name))
        .unwrap_or_else(|e| panic!("{name}: {e}"))
        .table
}

fn column<'a>(table: &'a Table, name: &str) -> &'a Column {
    table
        .columns
        .iter()
        .find(|c| c.name == name)
        .unwrap_or_else(|| panic!("no column {name}"))
}

#[test]
fn sensors_file_has_typed_columns_and_missing_values() {
    let table = load("sensors.csv");
    assert!(table.has_header);
    assert_eq!(table.row_count, 240);
    assert_eq!(column(&table, "time").kind, ColumnKind::DateTime);
    assert_eq!(column(&table, "temp").kind, ColumnKind::Numeric);
    assert_eq!(column(&table, "temp").missing_count, 2);
    assert_eq!(column(&table, "humidity").missing_count, 1);
    assert_eq!(column(&table, "site").kind, ColumnKind::Text);
    assert!(!column(&table, "site").usable_as_x());
    assert_eq!(table.bad_cells.total(), 0);
    // 2026-09-27T00:00:00.000 as written (no time zone conversion).
    assert_eq!(column(&table, "time").values()[0], 1_790_467_200.0);
}

#[test]
fn headerless_files_get_generated_names() {
    let numeric = load("numeric_no_header.csv");
    assert!(!numeric.has_header);
    assert_eq!(numeric.row_count, 50);
    let names: Vec<&str> = numeric.columns.iter().map(|c| c.name.as_str()).collect();
    assert_eq!(names, ["Column 1", "Column 2", "Column 3"]);

    let dated = load("datetime_no_header.csv");
    assert!(!dated.has_header);
    assert_eq!(dated.row_count, 28);
    assert_eq!(dated.columns[0].kind, ColumnKind::DateTime);
}

#[test]
fn delimiters_are_detected() {
    assert_eq!(load("semicolon.csv").delimiter, Delimiter::Semicolon);
    assert_eq!(load("tab.tsv").delimiter, Delimiter::Tab);
    let single = load("single_column.csv");
    assert_eq!(single.columns.len(), 1);
    assert_eq!(single.row_count, 3);
}

#[test]
fn quoted_fields_and_line_breaks() {
    let table = load("quoted.csv");
    assert_eq!(table.row_count, 3);
    assert_eq!(column(&table, "label").kind, ColumnKind::Text);
    assert_eq!(column(&table, "value").values(), [1.5, 2.5, 3.5]);
}

#[test]
fn blank_lines_and_crlf_are_skipped() {
    let table = load("blank_lines.csv");
    assert_eq!(table.row_count, 3);
    assert_eq!(column(&table, "y").values(), [2.0, 4.0, 6.0]);
}

#[test]
fn every_missing_token_is_missing_not_bad() {
    let table = load("missing_tokens.csv");
    let b = column(&table, "b");
    assert_eq!(b.kind, ColumnKind::Numeric);
    assert_eq!(b.missing_count, 12);
    assert_eq!(b.bad_cell_count, 0);
    assert_eq!(b.values().last(), Some(&42.0));
}

#[test]
fn duplicate_and_empty_headers_are_disambiguated() {
    let table = load("duplicate_headers.csv");
    let names: Vec<&str> = table.columns.iter().map(|c| c.name.as_str()).collect();
    assert_eq!(names, ["temp", "temp (2)", "Column 3", "temp (3)"]);
}

#[test]
fn offsets_are_converted_to_utc() {
    let table = load("offset_times.csv");
    let t = column(&table, "t").values();
    assert_eq!(t[0], t[1]);
    assert_eq!(t[2] - t[0], 3600.5);
}
