//! Acceptance: reduced views never hide spikes or gaps (FR-010).

#![allow(clippy::unwrap_used, clippy::panic)] // test helpers may fail loudly

use std::path::Path;

use plot_twist_core::dataset::{
    Column, ColumnIndex, ColumnKind, Dataset, DatasetId, Delimiter, Table,
};
use plot_twist_core::session::{PlotConfig, PlotStyle};
use plot_twist_core::view::{RAW_LIMIT, ViewMode, ViewRequest, reduce};

const N: usize = 1_000_000;
const SPIKE_AT: usize = 654_321;

fn dataset(ys: Vec<f64>, xs: Vec<f64>) -> Dataset {
    let mut x = Column::new(ColumnIndex(0), "x".into(), ColumnKind::Numeric, Some(xs));
    x.min = Some(0.0);
    let y = Column::new(ColumnIndex(1), "y".into(), ColumnKind::Numeric, Some(ys));
    let table = Table {
        has_header: true,
        delimiter: Delimiter::Comma,
        row_count: N as u64,
        columns: vec![x, y],
        bad_cells: Default::default(),
    };
    Dataset::new(DatasetId::new(1), Path::new("mem.csv"), 0, table)
}

fn spiky() -> Dataset {
    let xs: Vec<f64> = (0..N).map(|i| i as f64).collect();
    let mut ys: Vec<f64> = (0..N).map(|i| (i as f64 / 5000.0).sin()).collect();
    ys[SPIKE_AT] = 100.0;
    ys[SPIKE_AT + 10] = -100.0;
    ys[300_000] = f64::NAN;
    dataset(ys, xs)
}

fn view(ds: &Dataset, style: PlotStyle, x_min: f64, x_max: f64) -> (ViewMode, Vec<f64>) {
    let plot = PlotConfig {
        dataset_id: ds.id,
        x: "x".into(),
        y: vec!["y".into()],
        style,
    };
    let request = ViewRequest {
        dataset_id: ds.id,
        x_min,
        x_max,
        y_min: -150.0,
        y_max: 150.0,
        width_px: 1200,
        height_px: 600,
    };
    let mut payload = reduce(ds, &plot, &request).unwrap();
    let series = payload.series.remove(0);
    (series.mode, series.points)
}

fn ys(points: &[f64]) -> impl Iterator<Item = f64> + '_ {
    points.as_chunks::<2>().0.iter().map(|p| p[1])
}

#[test]
fn spikes_survive_line_reduction_at_every_zoom() {
    let ds = spiky();
    let spike = SPIKE_AT as f64;
    for half_width in [N as f64, 200_000.0, 20_000.0, 5_000.0, 500.0] {
        let (mode, points) = view(&ds, PlotStyle::Line, spike - half_width, spike + half_width);
        assert!(points.len() / 2 <= 2 * RAW_LIMIT.max(1200 * 4) + 4);
        assert!(
            ys(&points).any(|y| y == 100.0),
            "max spike lost at ±{half_width} ({mode:?})"
        );
        assert!(
            ys(&points).any(|y| y == -100.0),
            "min spike lost at ±{half_width}"
        );
    }
}

#[test]
fn full_view_is_reduced_and_small_view_is_raw() {
    let ds = spiky();
    assert_eq!(
        view(&ds, PlotStyle::Line, 0.0, N as f64).0,
        ViewMode::Reduced
    );
    assert_eq!(view(&ds, PlotStyle::Line, 1000.0, 2000.0).0, ViewMode::Raw);
}

#[test]
fn missing_values_break_the_line() {
    let ds = spiky();
    let (_, zoomed) = view(&ds, PlotStyle::Line, 299_990.0, 300_010.0);
    assert!(zoomed.iter().any(|v| v.is_nan()), "gap missing in raw view");
    let (_, full) = view(&ds, PlotStyle::Line, 0.0, N as f64);
    assert!(
        full.iter().any(|v| v.is_nan()),
        "gap missing in reduced view"
    );
}

#[test]
fn scatter_reduction_keeps_every_inked_pixel() {
    let ds = spiky();
    let (mode, points) = view(&ds, PlotStyle::Scatter, 0.0, N as f64);
    assert_eq!(mode, ViewMode::Reduced);
    assert!(ys(&points).any(|y| y == 100.0));
    assert!(ys(&points).any(|y| y == -100.0));
    assert!(points.len() / 2 <= 1200 * 600);
}

#[test]
fn unsorted_x_uses_an_envelope_that_keeps_extremes() {
    let xs: Vec<f64> = (0..N).map(|i| ((i * 7919) % N) as f64).collect();
    let mut values: Vec<f64> = vec![0.0; N];
    values[SPIKE_AT] = 100.0;
    let ds = dataset(values, xs);
    let (mode, points) = view(&ds, PlotStyle::Line, 0.0, N as f64);
    assert_eq!(mode, ViewMode::Reduced);
    assert!(ys(&points).any(|y| y == 100.0));
}
