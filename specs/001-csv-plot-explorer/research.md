# Research: CSV Plot Explorer

**Feature**: [spec.md](spec.md) | **Plan**: [plan.md](plan.md) | **Date**: 2026-09-27

Decisions marked **(user)** were made by the project owner during planning; the rest are
engineering decisions made to satisfy the spec and constitution.

## R1. Desktop shell — Tauri 2 **(user)**

- **Decision**: Tauri 2 with the system WebView2 runtime (preinstalled on Windows 11,
  evergreen-installable on Windows 10).
- **Rationale**: Rust backend + TypeScript UI in one small (~5–10 MB) installer; mature on
  Windows; matches the constitution's stack constraint.
- **Alternatives**: Electron + Rust sidecar (≥100 MB, two processes); egui (conflicts with
  the TypeScript-frontend constraint).

## R2. Frontend framework — React + Vite + TypeScript **(user)**

- **Decision**: React 19, Vite, TypeScript `strict`, ESLint (typescript-eslint strict) and
  Prettier; npm as package manager.
- **Rationale**: Most widely used framework and tooling — the "well-trodden path" the
  constitution requires for npm dependencies; largest contributor pool.
- **Alternatives**: Svelte 5, Vue 3, SolidJS (smaller ecosystems).

## R3. Plot rendering — custom renderer on PixiJS v8 **(user)**

- **Decision**: Build our own plot component. PixiJS v8 (WebGL2) draws series; d3-scale
  and d3-time/d3-time-format compute axis ticks and time labels; axis labels, legend, and
  hover readout are DOM overlays styled by theme CSS variables.
  - Lines: instanced segment quads in a custom Pixi `Mesh`/shader. The data→screen
    transform is a shader uniform, so pan/zoom never rebuilds geometry and line width stays
    constant in pixels.
  - Scatter: instanced quads with a circular signed-distance fragment shader.
- **Rationale**: The owner wants control below the level of Plotly-style libraries, as a
  foundation for capabilities beyond them. Uniform-driven transforms make 60 fps pan/zoom
  independent of point count.
- **Alternatives**: uPlot, ECharts, Plotly.js (higher-level, less control); raw WebGL2
  without Pixi (more boilerplate: context loss, resize, batching).

## R4. Keeping 1M points smooth — backend view reduction + frontend transform

- **Decision**:
  1. The frontend keeps the last reduced "view payload" on the GPU and, during a drag or
     wheel gesture, only updates the transform uniform (no backend round-trip per frame).
  2. After the gesture settles (debounce ~50 ms) or at most every 100 ms during continuous
     motion, the frontend requests a new payload for the current view.
  3. The backend reduces each series for the requested x/y range and pixel size:
     - If ≤ 20,000 points of a series are visible → send them raw (in file order).
     - **Line style**: min/max-per-pixel-column reduction (M4-style: first, min, max, last
       per column, in order), which preserves spikes and extremes (FR-010).
     - **Scatter style**: 2D pixel-grid occupancy — at most one representative point per
       occupied pixel cell, so every pixel that would be inked is still inked.
     - Non-monotonic X in line style above the raw threshold: per-pixel-column min/max
       envelope (file-order connectivity is shown faithfully once zoomed to ≤ 20,000
       visible points).
  4. Reduction is a linear scan of the visible range (no precomputed pyramid). 1M f64s per
     series scan in ~1 ms; 10 series well under the 100 ms re-query cadence.
- **Rationale**: Meets SC-002 without per-frame backend work; keeps full datasets out of
  the WebView (Principle I); YAGNI on multi-resolution pyramids until benchmarks demand
  them.
- **Alternatives**: LTTB (can drop extremes — violates FR-010); precomputed min/max
  pyramid (added complexity, not yet needed); sending all points to the GPU (1M × 10 × 16 B
  = 160 MB into the WebView; violates Principle I).

## R5. CSV parsing

- **Decision**: `csv` crate (`ByteRecord`, `flexible(true)` so we detect field-count
  mismatches ourselves and report them with line numbers). Numbers parsed with std
  `f64::from_str` (Eisel-Lemire, fast); ISO 8601 date-times parsed with `jiff`. Columns
  stored as `Vec<f64>`; date/time as f64 seconds since 1970-01-01 on a naive timeline
  (offset values converted to UTC; no-offset values kept as written). Missing = `NaN`.
- Delimiter sniffing: count `,` `;` `\t` outside quotes on the first 20 lines; pick the
  candidate with the most consistent non-zero count (ties → comma).
- Header detection: first record all-numeric → no header, columns named `Column N`.
- Encoding: UTF-8 validated per record; BOM stripped; invalid bytes → `INVALID_ENCODING`
  with line number.
- Single-threaded first; a ~110 MB 1M×10 file needs only ~40 MB/s to meet the 3 s budget.
  Parallelism (rayon over chunks) only if the perf check fails.
- **Alternatives**: Polars (heavy dependency, loses precise per-cell error reporting);
  `chrono` (older API; `jiff` is the modern, well-maintained choice).

## R6. Bad-cell storage

- **Decision**: Record every bad cell (line, column index, original text) in a compact
  arena: fixed-size entries plus one shared text buffer; texts longer than 64 characters
  are stored truncated with `…`. Totals are exact. The GUI and API page through them
  (`offset`/`limit`).
- **Rationale**: FR-011a requires listing each bad cell; a pathological file can have
  millions, so storage must be compact and access paged.

## R7. One operation layer, three clients (Principle I)

- **Decision**: `plot-twist-core` exposes a `Session` with all data operations
  (`load_file`, `state`, `set_plot`, `bad_cells`, `view`). Tauri commands and the HTTP API
  are thin adapters over the same `Session` (shared via `Arc<RwLock<…>>`). Every state
  change publishes a `SessionEvent` on a broadcast channel; the Tauri layer forwards it to
  the WebView as a `session-changed` event so API-driven changes appear in the GUI
  (FR-015/FR-016).
- Loads run on a blocking worker thread; the session keeps serving reads during a load
  (FR-005). A new load replaces the dataset only on success (FR-004).

## R8. Local HTTP API **(user: plain HTTP+JSON, fixed port + discovery file)**

- **Decision**: `axum` server bound to `127.0.0.1` only (FR-017). Default port 47811,
  configurable in settings; if busy, bind an OS-assigned port. The actual address is always
  written to `%LOCALAPPDATA%\plot-twist\api.json` on startup and removed on clean exit.
  Versioned under `/v1`. Contract: [contracts/local-api.md](contracts/local-api.md).
- **Single instance**: `tauri-plugin-single-instance` — a second launch focuses the
  existing window, so there is never ambiguity about which app a script talks to.

## R9. Rust → TypeScript types (Principle II)

- **Decision**: `ts-rs` derives TypeScript definitions for every type crossing the Tauri
  boundary, exported to `ui/src/backend/generated/`. CI regenerates and fails on any diff.
- **Alternatives**: `tauri-specta` (also generates command wrappers, but its Tauri 2
  support has lagged); hand-written types (drift risk — forbidden by Principle II).

## R10. Errors and logging (Principle V)

- **Decision**: One `thiserror` enum `PtError` in core; each variant has a stable
  `SCREAMING_SNAKE` code, an English message, and a hint, carried in a serializable
  `ErrorReport` (see [contracts/error-codes.md](contracts/error-codes.md)). Workspace
  Clippy lints deny `unwrap_used`, `expect_used`, `panic`, `todo`, `unreachable` (allowed in
  tests via `clippy.toml`). `tracing` + `tracing-appender` write JSON-lines logs to
  `%LOCALAPPDATA%\plot-twist\logs\` (daily rotation, 7 files kept).
- **Message ownership** (constitution v1.0.2, Principle V): message and hint text is
  authored once in Rust; the GUI branches on the code for presentation (placement, actions
  such as "remove from recent files") and displays that text, so the GUI and API always
  match (FR-016).

## R11. Settings and recent files

- **Decision**: A small JSON settings file in the Tauri app config dir
  (`%APPDATA%\plot-twist\settings.json`): theme (`system|light|dark`), API port, recent
  files (max 10, most recent first). Owned by a core `settings` module; corrupted or
  missing file → defaults plus a logged warning (never a crash).
- Theme: CSS variables; `system` follows `prefers-color-scheme`, which WebView2 derives from
  the Windows app-mode setting and updates live.

## R12. Testing and performance verification (Principles III, IV)

- **Kept tests** (acceptance-critical only):
  - Core integration tests over a fixture corpus of valid and malformed CSVs (each Story 2
    case, header detection, delimiters, date/time, missing tokens).
  - View-reduction faithfulness: a single-sample spike survives reduction at every zoom.
  - API acceptance tests: start the HTTP adapter over an in-process `Session` and replay
    Stories 1–3 end-to-end (also proves parity, SC-007).
  - Frontend: Vitest only for logic whose failure users would see (tick/time formatting,
    hover nearest-point).
- **Performance checks** (release mode, in CI on `windows-latest`): a `perf` test target
  generates a 1M×10 CSV, asserts load < 3 s, reduction query < 50 ms for 10 series, and
  process memory < 3× file size (`memory-stats`). Startup < 2 s: CI launches the built app
  with `PLOT_TWIST_EXIT_WHEN_READY=1` and times to first successful `/v1/health` after the
  window reports ready. 60 fps: measured manually for rendering changes and recorded in
  the PR (constitution).
- No GUI end-to-end automation in this feature (WebDriver on Windows is heavy); GUI
  behavior is validated through [quickstart.md](quickstart.md).

## R13. Toolchain

- **Decision**: Rust 2024 edition; toolchain pinned via `rust-toolchain.toml` (stable
  1.98); Node 24 LTS pinned via `.nvmrc` and `engines`. GitHub Actions on `windows-latest`.
