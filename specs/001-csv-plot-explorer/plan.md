# Implementation Plan: CSV Plot Explorer

**Branch**: `001-csv-plot-explorer` | **Date**: 2026-09-27 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `specs/001-csv-plot-explorer/spec.md`

## Summary

First vertical slice of plot-twist: load a CSV (numeric, ISO 8601 date/time, and text
columns; bad cells become NaN with a full warning list), plot one X against several Y
columns as line or scatter, and explore with pan, wheel zoom, box zoom, hover readout, and
legend toggling — at 60 fps with 1M points per series. Every data operation is one Rust
`Session` API used by both the Tauri GUI and a localhost HTTP+JSON API, so scripts can
load and plot into the open window.

Technical approach: Tauri 2 shell; Rust workspace with a Tauri-free `plot-twist-core`
(parsing, dataset, reduction, session, errors, settings) and a `plot-twist-api` axum
adapter; React + TypeScript UI with a custom PixiJS v8 plot renderer (instanced-segment
shaders, transform-uniform pan/zoom) fed by backend min/max view reduction. See
[research.md](research.md).

## Technical Context

**Language/Version**: Rust 1.98 stable (2024 edition, pinned in `rust-toolchain.toml`);
TypeScript 5.x (`strict`) on Node 24 LTS

**Primary Dependencies**: Rust — `tauri` 2, `tauri-plugin-dialog`,
`tauri-plugin-single-instance`, `axum`, `tokio`, `csv`, `jiff`, `serde`/`serde_json`,
`thiserror`, `tracing`/`tracing-appender`, `ts-rs`, `memory-stats` (perf tests only).
Frontend — `react` 19, `vite`, `@tauri-apps/api` 2, `pixi.js` 8, `d3-scale`, `d3-time`,
`d3-time-format`; dev: `typescript`, `eslint` + `typescript-eslint`, `prettier`, `vitest`.

**Storage**: Files only — user CSVs (read-only); `%APPDATA%\plot-twist\settings.json`;
`%LOCALAPPDATA%\plot-twist\api.json` and `logs\`.

**Testing**: `cargo test` (core integration tests over a CSV fixture corpus; API acceptance
tests over an in-process session); release-mode `perf` test target; Vitest for
user-visible frontend logic only.

**Target Platform**: Windows 10/11 x64, WebView2.

**Project Type**: Desktop application (Tauri: Rust backend + web frontend) with a local
HTTP API.

**Performance Goals**: 60 fps pan/zoom at 1M points/series; load 1M×10 CSV < 3 s; cold
start < 2 s; view reduction < 50 ms for 10 × 1M series.

**Constraints**: Memory < 3× file size; full datasets never cross into the WebView; API
bound to 127.0.0.1 only; no panics in non-test code.

**Scale/Scope**: One dataset at a time; up to ~1M rows × tens of columns in budget; ~3
screens' worth of UI (data panel, plot, settings).

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Gate | Pre-design | Post-design |
|-----------|------|-----------|-------------|
| I. Backend owns data; clients equal | All parsing/reduction in Rust; GUI and API share one operation layer; API changes appear in GUI | ✅ | ✅ `Session` in core; Tauri commands and HTTP handlers are adapters ([tauri-commands.md](contracts/tauri-commands.md) maps 1:1 to [local-api.md](contracts/local-api.md)); `session-changed` events; only reduced payloads reach the WebView. View state (zoom, series visibility) and app preferences (theme, recent list) are GUI-only, as the exemption allows. |
| II. Type safety | Newtypes/enums; TS strict, no `any`; generated TS types | ✅ | ✅ `DatasetId`, `ColumnIndex`, `LineNumber`, `ColumnKind`, `PlotStyle`; `ts-rs` generation with CI drift check. |
| III. Validate, keep what matters | Acceptance-level kept tests only | ✅ | ✅ Fixture-corpus, reduction-faithfulness, and API acceptance tests; no implementation-detail tests kept (R12). |
| IV. Performance budgets | Benchmarks in CI; FPS recorded in PR | ✅ | ✅ `perf` test target (load, reduction, memory); startup check in CI; FPS overlay for manual measurement. |
| V. Errors: closed vocabulary, no panics | Central error enum; stable codes on API; actionable GUI messages; lint-enforced no-panic; file logs | ✅ | ✅ [error-codes.md](contracts/error-codes.md); `PtError` → `ErrorReport`; Clippy deny list; `tracing` JSON logs. Per constitution v1.0.2, message text is authored once in Rust and shown identically by GUI and API (R10). |
| VI. Simplicity | No speculative abstractions | ✅ | ✅ Linear-scan reduction (no pyramid), single-threaded parser, no plugin system; parallelism/pyramids only if perf checks fail. Dataset IDs are an explicit spec requirement (FR-014a), not speculation. |
| VII. Code organization | Screaming top-level buckets; depth ≤ 3–4; functions < ~100 lines; files < ~500 lines | ✅ | ✅ See structure below. |
| Constraints | GPL-3.0-or-later compatible deps; Windows only; npm deps popular & maintained; localhost API | ✅ | ✅ All listed deps are MIT/Apache-2.0/BSD/ISC (GPL-compatible); npm deps are mainstream. |
| Workflow | fmt, clippy -D warnings, tsc strict, ESLint, Prettier; GitHub Actions on Windows | ✅ | ✅ `.github/workflows/ci.yml` planned. |

No violations — Complexity Tracking not needed.

## Project Structure

### Documentation (this feature)

```text
specs/001-csv-plot-explorer/
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   ├── local-api.md
│   ├── error-codes.md
│   └── tauri-commands.md
├── checklists/
└── tasks.md             # /speckit-tasks
```

### Source Code (repository root)

```text
Cargo.toml                    # workspace, shared lints
rust-toolchain.toml
clippy.toml
package.json                  # npm scripts: tauri, check, bindings
.nvmrc
.github/workflows/ci.yml

crates/
├── core/                     # plot-twist-core (no Tauri dependency)
│   ├── src/
│   │   ├── lib.rs
│   │   ├── dataset/          # Dataset, Column, ids, BadCellStore
│   │   ├── csv_import/       # sniffing, header detection, typed parsing, bad cells
│   │   ├── view/             # view reduction (raw / min-max / scatter occupancy)
│   │   ├── session/          # Session operations, SessionState, SessionEvent
│   │   ├── errors/           # PtError, ErrorReport, codes
│   │   └── settings/         # Settings load/save, recent files
│   ├── tests/                # fixture-corpus, reduction, session acceptance tests
│   │   └── perf.rs           # #[ignore] budgets test (release, CI)
│   └── examples/gen_csv.rs   # large-file generator for perf & manual checks
└── api/                      # plot-twist-api: axum adapter over Session
    ├── src/
    │   ├── lib.rs
    │   ├── routes/           # /v1 handlers
    │   └── discovery/        # port binding, api.json
    └── tests/                # HTTP acceptance tests (Stories 1–3 via API)

src-tauri/                    # plot-twist app (Tauri shell)
├── Cargo.toml
├── tauri.conf.json
├── capabilities/
└── src/
    ├── main.rs
    ├── commands/             # Tauri command adapters over Session
    ├── events/               # SessionEvent → WebView forwarding
    └── startup/              # logging, settings, API server, single instance

ui/                           # React + TypeScript frontend (Vite root)
├── index.html
└── src/
    ├── main.tsx
    ├── app/                  # shell layout, menus, theme, recent files, status bar
    ├── data-panel/           # file open/drop, column picker, warnings list
    ├── plot/                 # Pixi renderer, shaders, axes, legend, hover, gestures
    └── backend/              # typed command/event wrappers
        └── generated/        # ts-rs output (do not edit)

fixtures/
├── csv/                      # valid + malformed sample files
└── scripts/load_and_plot.py  # stdlib-only API example
```

**Structure Decision**: Tauri desktop layout with a Cargo workspace. The domain lives in a
Tauri-free `crates/core` so it can be tested and benchmarked without building the shell;
`crates/api` and `src-tauri` are thin adapters (Principle I). Rust top-level buckets
(`dataset`, `csv_import`, `view`, `session`, `errors`, `settings`) and frontend buckets
(`app`, `data-panel`, `plot`, `backend`) are the screaming-architecture split
(Principle VII).

## Complexity Tracking

Not applicable — Constitution Check passes without violations.
