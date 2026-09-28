//! Performance budgets (constitution Principle IV; SC-001, SC-004). Release mode only:
//! `cargo test -p plot-twist-core --release --test perf -- --ignored --nocapture`

#![allow(clippy::unwrap_used, clippy::panic)] // test helpers may fail loudly

use std::io::{BufWriter, Write};
use std::time::{Duration, Instant};

use plot_twist_core::session::{Origin, PlotConfig, PlotStyle, Session};
use plot_twist_core::settings::SettingsStore;
use plot_twist_core::view::ViewRequest;

const ROWS: u64 = 1_000_000;
const COLS: usize = 10;
const LOAD_BUDGET: Duration = Duration::from_secs(3);
const VIEW_BUDGET: Duration = Duration::from_millis(50);
const MEMORY_RATIO_BUDGET: f64 = 3.0;

fn write_csv(path: &std::path::Path) {
    let mut out = BufWriter::new(std::fs::File::create(path).unwrap());
    let header: Vec<String> = (0..COLS).map(|c| format!("c{c}")).collect();
    writeln!(out, "{}", header.join(",")).unwrap();
    for row in 0..ROWS {
        write!(out, "{row}").unwrap();
        for c in 1..COLS {
            write!(
                out,
                ",{:.4}",
                ((row as f64) / 997.0 + c as f64).sin() * 1234.5
            )
            .unwrap();
        }
        writeln!(out).unwrap();
    }
    out.flush().unwrap();
}

fn resident_bytes() -> f64 {
    memory_stats::memory_stats().map_or(0.0, |m| m.physical_mem as f64)
}

#[test]
#[ignore = "release-mode performance budget; run explicitly"]
fn one_million_rows_meet_budgets() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("big.csv");
    write_csv(&path);
    let file_size = std::fs::metadata(&path).unwrap().len() as f64;

    let baseline = resident_bytes();
    let session = Session::new(SettingsStore::in_memory());
    let started = Instant::now();
    let loaded = session.load_file(&path, Origin::Gui).unwrap();
    let load_time = started.elapsed();
    let memory_ratio = (resident_bytes() - baseline) / file_size;

    let y: Vec<String> = (1..COLS).map(|c| format!("c{c}")).collect();
    let plot = PlotConfig {
        dataset_id: loaded.dataset.id,
        x: "c0".into(),
        y,
        style: PlotStyle::Line,
    };
    session.set_plot(plot, Origin::Gui).unwrap();
    let request = ViewRequest {
        dataset_id: loaded.dataset.id,
        x_min: 0.0,
        x_max: ROWS as f64,
        y_min: -2000.0,
        y_max: 2000.0,
        width_px: 1600,
        height_px: 900,
    };
    session.view(&request).unwrap(); // warm up
    let started = Instant::now();
    session.view(&request).unwrap();
    let view_time = started.elapsed();

    println!(
        "file {:.0} MB | load {load_time:?} (budget {LOAD_BUDGET:?}) | view {view_time:?} \
         (budget {VIEW_BUDGET:?}) | memory {memory_ratio:.2}x file (budget {MEMORY_RATIO_BUDGET}x)",
        file_size / 1e6
    );
    assert!(load_time < LOAD_BUDGET, "load took {load_time:?}");
    assert!(view_time < VIEW_BUDGET, "view reduction took {view_time:?}");
    assert!(
        memory_ratio < MEMORY_RATIO_BUDGET,
        "memory {memory_ratio:.2}x file size"
    );
}
