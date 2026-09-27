# Quickstart: Validate CSV Plot Explorer

End-to-end validation guide for [spec.md](spec.md). Contracts:
[local-api.md](contracts/local-api.md), [error-codes.md](contracts/error-codes.md).

## Prerequisites

- Windows 10/11 x64 with WebView2 runtime (preinstalled on Windows 11).
- Rust (version pinned by `rust-toolchain.toml`) with the MSVC toolchain and Visual Studio
  Build Tools ("Desktop development with C++").
- Node.js 24 LTS and npm.
- Python 3 (only for the API example script).

## Setup

```bash
npm ci
```

## Automated checks (what CI runs)

```bash
cargo fmt --all --check
```

```bash
cargo clippy --workspace --all-targets -- -D warnings
```

```bash
cargo test --workspace
```

```bash
cargo test -p plot-twist-core --release --test perf -- --ignored --nocapture
```

```bash
npm run check
```

`npm run check` runs `tsc --noEmit`, ESLint, Prettier `--check`, Vitest, and the
generated-bindings drift check.

Expected: all pass. The perf test prints load time, reduction time, and memory ratio for a
generated 1M×10 file and fails if any budget (SC-001, SC-004) is exceeded.

## Run the app

```bash
npm run tauri dev
```

Sample files are in `fixtures/csv/` (valid and malformed cases). A 1M-row file can be
generated with:

```bash
cargo run -p plot-twist-core --release --example gen_csv -- fixtures/generated/big.csv 1000000 10
```

## Manual scenarios

### Story 1 — explore a file

1. Open `fixtures/csv/sensors.csv` via File → Open. Expect file name, row count, and columns
   with type and missing counts.
2. Pick X = `time`, Y = `temp` and `pressure`. Expect two colored series, legend, labeled
   axes, time-formatted X ticks.
3. Drag → box zoom. Shift+drag (or middle-drag) → pans. Wheel → zooms at pointer. Double-click or
   "Reset view" → fits visible series.
4. Hover a point → series name, exact X (full date-time) and Y.
5. Click `pressure` in the legend → hidden; reset view ignores it; click again → shown.
6. Toggle Line/Scatter → redraws without reload.
7. Drag `fixtures/csv/numeric_no_header.csv` onto the window → replaces dataset; columns
   named `Column 1…N`; plot selection cleared. Drop two files → "drop a single file".
8. File → Recent → first entry reopens it. Rename a recent file on disk, choose it → "file
   not found" with an offer to remove it.
9. Settings → Appearance: Light / Dark / System; System follows Windows app mode live;
   choice survives restart.

### Story 2 — malformed files

| File | Expected |
|------|----------|
| `bad_cells.csv` | Loads; warning "1 bad cell" listing line 1,204, `temp`, `abc`; gap in line plot |
| `ragged.csv` | `FIELD_COUNT_MISMATCH` with line, expected vs actual; previous dataset still shown |
| `empty.csv` / `header_only.csv` | `EMPTY_FILE` / `NO_DATA_ROWS` |
| `latin1.csv` | `INVALID_ENCODING` with line |
| `decimal_comma.csv` | Loads; `1,5`-style values listed as bad cells |

### Story 3 — local API

With the app running:

```bash
python fixtures/scripts/load_and_plot.py fixtures/csv/sensors.csv time temp pressure
```

Expect the window to show the file and plot; the script prints the dataset id and
columns. Run it against `ragged.csv` → the script prints the `FIELD_COUNT_MISMATCH`
report and the window shows the same error.

### Performance (record in PR)

- Load `fixtures/generated/big.csv`: loaded in < 3 s (status bar shows load time).
- Plot 1 X + 3 Y (1M points each); pan and zoom continuously; frame rate ≥ 60 fps
  (Settings → Diagnostics → show FPS overlay).
- Cold start to interactive window < 2 s (CI also checks).
