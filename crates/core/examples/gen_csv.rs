//! Writes a large numeric CSV for performance checks.
//!
//! Usage: `cargo run -p plot-twist-core --release --example gen_csv -- <path> <rows> <cols>`
//! Column 1 is `x` (0, 1, 2, …); the others are smooth waves. Every 100,000th row of
//! column 2 is a single-sample spike, so reduction faithfulness is easy to see.

use std::io::{BufWriter, Write};
use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().collect();
    let (Some(path), Some(rows), Some(cols)) = (
        args.get(1),
        args.get(2).and_then(|r| r.parse::<u64>().ok()),
        args.get(3).and_then(|c| c.parse::<usize>().ok()),
    ) else {
        eprintln!("usage: gen_csv <path> <rows> <cols>");
        return ExitCode::FAILURE;
    };
    match write(path, rows, cols.max(2)) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("gen_csv: {error}");
            ExitCode::FAILURE
        }
    }
}

fn write(path: &str, rows: u64, cols: usize) -> std::io::Result<()> {
    if let Some(dir) = std::path::Path::new(path).parent() {
        std::fs::create_dir_all(dir)?;
    }
    let mut out = BufWriter::new(std::fs::File::create(path)?);
    let header: Vec<String> = std::iter::once("x".to_owned())
        .chain((1..cols).map(|c| format!("s{c}")))
        .collect();
    writeln!(out, "{}", header.join(","))?;
    for row in 0..rows {
        write!(out, "{row}")?;
        for c in 1..cols {
            let t = row as f64 / 1000.0 + c as f64;
            let spike = c == 1 && row % 100_000 == 50_000;
            let value = if spike {
                1000.0
            } else {
                (t * (1.0 + c as f64 * 0.1)).sin() * 10.0
            };
            write!(out, ",{value:.4}")?;
        }
        writeln!(out)?;
    }
    out.flush()
}
