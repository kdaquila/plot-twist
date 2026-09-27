---

description: "Task list for CSV Plot Explorer"
---

# Tasks: CSV Plot Explorer

**Input**: Design documents from `specs/001-csv-plot-explorer/`

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/, quickstart.md

**Tests**: Kept tests are limited to acceptance-critical behavior (constitution Principle
III, research R12). Authors may write throwaway tests freely while implementing and must
delete them before merge.

**Organization**: Tasks are grouped by user story so each story can be implemented and
validated independently.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (US1, US2, US3)

## Path Conventions

Tauri desktop layout per plan.md: `crates/core/` (domain), `crates/api/` (HTTP adapter),
`src-tauri/` (shell), `ui/` (React frontend), `fixtures/` (sample data).

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Workspace, toolchains, quality gates, CI

- [ ] T001 Create the Cargo workspace in `Cargo.toml` (members `crates/core`, `crates/api`, `src-tauri`; edition 2024; `resolver = "3"`; `[workspace.lints.clippy]` deny `unwrap_used`, `expect_used`, `panic`, `todo`, `unreachable`, `undocumented_unsafe_blocks` (so `unsafe` is allowed only with a `// SAFETY:` comment)), plus `rust-toolchain.toml` (channel `1.98`, components `rustfmt`, `clippy`), `clippy.toml` (`allow-unwrap-in-tests`, `allow-expect-in-tests`, `allow-panic-in-tests` = true), and `rustfmt.toml` (`max_width = 100`)
- [ ] T002 [P] Scaffold `crates/core` as `plot-twist-core` (`crates/core/Cargo.toml` with `csv`, `jiff`, `serde`, `serde_json`, `thiserror`, `tracing`, `tokio` (sync), `ts-rs`; dev-deps `memory-stats`, `tempfile`; `lints.workspace = true`) and `crates/core/src/lib.rs` declaring modules `dataset`, `csv_import`, `view`, `session`, `errors`, `settings`
- [ ] T003 [P] Scaffold `crates/api` as `plot-twist-api` (`crates/api/Cargo.toml` with `axum`, `tokio` (rt-multi-thread, net), `serde`, `serde_json`, `tracing`, `plot-twist-core`; dev-deps `reqwest` (json), `tempfile`) and `crates/api/src/lib.rs` declaring modules `routes`, `discovery`
- [ ] T004 Scaffold the Tauri 2 app in `src-tauri/` (`Cargo.toml` package `plot-twist` with `tauri`, `tauri-plugin-dialog`, `tauri-plugin-single-instance`, `tracing`, `tracing-subscriber` (json), `tracing-appender`, `tokio`, `plot-twist-core`, `plot-twist-api`; `build.rs`; `tauri.conf.json` with productName `plot-twist`, identifier `io.github.kdaquila.plottwist`, devUrl `http://localhost:5173`, frontendDist `../dist`, one 1280×800 window titled `plot-twist`, `dragDropEnabled: true`; `capabilities/default.json` granting core + dialog permissions; `src/main.rs`; placeholder icons generated from `src-tauri/icons/icon.svg` with `npx tauri icon`)
- [ ] T005 Scaffold the frontend: root `package.json` (name `plot-twist`, `engines.node >=24`, scripts `dev`, `build`, `tauri`, `lint`, `format`, `format:check`, `typecheck`, `test`, `bindings`, `check` = typecheck + lint + format:check + test + bindings drift check), `vite.config.ts` (root `ui`, `build.outDir` `../dist`, port 5173 strict), `tsconfig.json` (`strict`, `noUncheckedIndexedAccess`, `exactOptionalPropertyTypes`), `ui/index.html`, `ui/src/main.tsx`; install `react`, `react-dom`, `@tauri-apps/api`, `@tauri-apps/plugin-dialog`, `pixi.js@8`, `d3-scale`, `d3-time`, `d3-time-format` and dev deps `typescript`, `vite`, `@vitejs/plugin-react`, `@tauri-apps/cli`, `vitest`, `eslint`, `typescript-eslint`, `eslint-plugin-react-hooks`, `prettier`, `@types/react`, `@types/react-dom`, `@types/d3-scale`, `@types/d3-time`, `@types/d3-time-format`
- [ ] T006 [P] Add `eslint.config.js` (typescript-eslint `strictTypeChecked`, react-hooks rules, `@typescript-eslint/no-explicit-any: error`, ignore `ui/src/backend/generated/`), `.prettierrc.json`, `.prettierignore`, and `.nvmrc` (`24`)
- [ ] T007 [P] Extend root `.gitignore` with `target/`, `node_modules/`, `dist/`, `fixtures/generated/`, `src-tauri/gen/`
- [ ] T008 [P] Create `.github/workflows/ci.yml` (on `pull_request` and push to `main`; `windows-latest`; cache cargo + npm; steps: `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, `npm ci`, `npm run check`, perf test (T056), startup check (T057))

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core types, error vocabulary, session skeleton, shell wiring, generated bindings

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [ ] T009 [P] Implement the error vocabulary in `crates/core/src/errors/mod.rs` and `crates/core/src/errors/report.rs`: `PtError` (`thiserror`) with one variant per code in `contracts/error-codes.md` (load, plot, request failures, and `BadCell` warning kind), `fn code(&self) -> &'static str`, English `message()` and the exact `hint()` text from the hints table, and a serializable `ErrorReport { code, message, hint, line: Option<u64>, column: Option<String>, value: Option<String>, details: Option<serde_json::Value> }` deriving `ts_rs::TS`; offending values truncated to 64 chars with `…`
- [ ] T010 [P] Implement newtypes in `crates/core/src/dataset/ids.rs`: `DatasetId(u64)` serialized as `"ds-N"` (and parsed back), `ColumnIndex(u32)`, `LineNumber(u64)` (1-based, header is line 1), all deriving `ts_rs::TS`
- [ ] T011 Implement dataset types in `crates/core/src/dataset/mod.rs` and `crates/core/src/dataset/column.rs`: `Dataset { id, path, file_name, file_size_bytes, row_count (≥ 1), has_header, delimiter: Comma|Semicolon|Tab, columns }`, `ColumnKind = Numeric|DateTime|Text`, `Column { index, name (unique), kind, missing_count, bad_cell_count, values: Option<Vec<f64>> }` with `usable_as_x = kind ∈ {Numeric, DateTime}`, `usable_as_y = kind == Numeric`, and serializable `DatasetSummary`/`ColumnInfo` (no values, includes `bad_cell_total`) deriving `ts_rs::TS`
- [ ] T012 [P] Implement settings in `crates/core/src/settings/mod.rs`: `Settings { theme: System|Light|Dark (default System), api_port: u16 (default 47811), recent_files: Vec<PathBuf> (≤ 10, most recent first, de-duplicated) }`; `load(path)` returns defaults and logs a warning on missing/corrupt file (never errors); `save(path)`; `record_recent(path)`; `remove_recent(path)`
- [ ] T013 Implement the session skeleton in `crates/core/src/session/mod.rs`, `crates/core/src/session/events.rs`, and `crates/core/src/session/plot.rs`: `Session` holding `Option<Arc<Dataset>>`, `Option<PlotConfig>`, `loading: Option<PathBuf>`, next `DatasetId`, and a `tokio::sync::broadcast::Sender<SessionEvent>`; `SessionState` (per data-model.md); `SessionEvent = LoadStarted{path} | DatasetLoaded{summary, warnings_total} | LoadFailed{report} | PlotChanged{plot}` each with `origin: Gui|Api`; `PlotConfig { dataset_id, x, y, style: Line|Scatter }`; `LoadResult { dataset: DatasetSummary, bad_cells_preview: Vec<BadCell> (≤ 100) }`; `fn state()`; all ts-rs derived
- [ ] T014 Add ts-rs export: a test `crates/core/tests/export_bindings.rs` (and `ts-rs` `export_to = "../../ui/src/backend/generated/"`) that writes all boundary types; npm script `bindings` runs it; `check` fails if `git diff --exit-code ui/src/backend/generated` is non-empty after regeneration
- [ ] T015 Implement shell startup in `src-tauri/src/startup/mod.rs`, `src-tauri/src/startup/logging.rs`, `src-tauri/src/main.rs`: `tracing` JSON-lines logs to `%LOCALAPPDATA%\plot-twist\logs\` (daily rotation, keep 7); `tauri-plugin-single-instance` that focuses the existing window on second launch; load `Settings` from the app config dir (`%APPDATA%\plot-twist\settings.json`); manage `Arc<RwLock<Session>>` and settings as Tauri state; no `unwrap`/`expect` (startup failures logged and shown)
- [ ] T016 Implement event forwarding in `src-tauri/src/events/mod.rs`: subscribe to the session broadcast and emit each `SessionEvent` to the WebView as `session-changed`
- [ ] T017 [P] Implement typed frontend backend wrappers in `ui/src/backend/commands.ts` and `ui/src/backend/events.ts` using only generated types from `ui/src/backend/generated/` (`invoke` wrappers returning typed promises; rejections normalized to `ErrorReport`; `onSessionChanged`, `onApiStatus` listeners)
- [ ] T018 Implement the app shell in `ui/src/app/App.tsx`, `ui/src/app/layout.css`, `ui/src/app/theme.ts`, `ui/src/app/theme.css`: layout with data panel (left), plot area (center), status bar (bottom); theme CSS variables for light and dark; `System` follows `prefers-color-scheme` live; Light/Dark/System control in a settings menu persisted via `set_theme`; `get_settings`/`set_theme` Tauri commands in `src-tauri/src/commands/settings.rs`

**Checkpoint**: Foundation ready — app launches to an empty shell with theming; bindings generate

---

## Phase 3: User Story 1 — Open a CSV and explore it as a plot (Priority: P1) 🎯 MVP

**Goal**: Load a valid CSV and explore X vs. multiple Y as line/scatter with pan, wheel zoom, box zoom, hover, legend toggling, keyboard control — at 60 fps for 1M points/series

**Independent Test**: Open `fixtures/csv/sensors.csv`, plot `time` vs. `temp` and `pressure`, use every interaction in quickstart.md Story 1

### Data and fixtures

- [ ] T019 [P] [US1] Create valid fixtures in `fixtures/csv/`: `sensors.csv` (ISO 8601 `time` column with fractional seconds, `temp`, `pressure`, `humidity`, a text `site` column, a few `NA` cells), `numeric_no_header.csv`, `datetime_no_header.csv` (first column date-times, no header), `semicolon.csv`, `tab.tsv`, `quoted.csv` (quoted delimiters and a quoted line break), `missing_tokens.csv` (every token: empty, `NaN`, `NA`, `N/A`, `#N/A`, `null`, `None` in mixed case, `inf`, `-inf`), `duplicate_headers.csv`, `single_column.csv`, `blank_lines.csv` (CRLF, blank lines mid-file and at end), `offset_times.csv` (`Z` and `+02:00`)
- [ ] T020 [P] [US1] Implement `crates/core/examples/gen_csv.rs`: `gen_csv <path> <rows> <cols>` writes a header plus deterministic numeric data (one column with occasional single-sample spikes)

### Import (happy path)

- [ ] T021 [P] [US1] Implement delimiter sniffing in `crates/core/src/csv_import/sniff.rs`: count `,` `;` `\t` outside quotes on the first 20 non-blank lines; choose the candidate with the most consistent non-zero count; ties or ambiguity → comma; none found → single column
- [ ] T022 [P] [US1] Implement cell classification in `crates/core/src/csv_import/cells.rs`: trim spaces; missing tokens (empty, `nan`, `na`, `n/a`, `#n/a`, `null`, `none`, case-insensitive) → missing; numbers via `f64::from_str` (leading `+`/`-`, `.` decimals, scientific notation; no thousands separators); non-finite → missing; ISO 8601 date, date-time (`T` or space separator), fractional seconds, optional `Z`/`±HH:MM` via `jiff` → f64 seconds since 1970-01-01 (offset values converted to UTC, no-offset kept as written, date-only = midnight); report whether a date-time had an offset
- [ ] T023 [P] [US1] Implement header handling in `crates/core/src/csv_import/header.rs`: first record is data (no header) when every field is a number or ISO date/time → names `Column 1…N`; otherwise header names, empty names → `Column N`, duplicates → `name (2)`, `name (3)`…
- [ ] T024 [US1] Implement the streaming reader in `crates/core/src/csv_import/mod.rs` and `crates/core/src/csv_import/columns.rs`: `csv::ReaderBuilder` with sniffed delimiter, `flexible(true)`, BOM stripped; skip fully blank records (still counted in line numbers); track each record's first line as `LineNumber`; set each column's `kind` from its first value that is neither empty nor a missing token (no such value → `Numeric`); append parsed values to `Vec<f64>` buffers (text columns store none); cells not matching the column kind (including offset/no-offset mismatch against the first value) → NaN and counted in `bad_cell_count`/`missing_count`; `try_reserve` for buffers; returns `Dataset`
- [ ] T025 [US1] Implement `Session::load_file(path, origin)` in `crates/core/src/session/load.rs`: refuse with `LOAD_IN_PROGRESS` if loading; set `loading`, emit `LoadStarted`; parse on a blocking worker (`tokio::task::spawn_blocking`) without holding the session lock, so reads and view queries keep working during the load; on success assign a new `DatasetId`, replace dataset atomically, clear plot, emit `DatasetLoaded`, and record the path in recent files; on failure keep previous dataset and plot, emit `LoadFailed`; always clear `loading`
- [ ] T026 [US1] Implement `Session::set_plot(config, origin)` in `crates/core/src/session/plot.rs` enforcing data-model rules in order: `dataset_id` equals loaded id else `UNKNOWN_DATASET`/`STALE_DATASET`; `x` exists (`UNKNOWN_COLUMN`) and `usable_as_x` (`COLUMN_NOT_USABLE_AS_X`); `y` non-empty (`NO_Y_COLUMNS`), unique (`DUPLICATE_Y_COLUMN`), each exists and `usable_as_y` (`COLUMN_NOT_USABLE_AS_Y`), not containing `x` (`X_ALSO_Y`); on success store and emit `PlotChanged`; on failure plot unchanged

### View reduction

- [ ] T027 [P] [US1] Implement view reduction in `crates/core/src/view/mod.rs`, `crates/core/src/view/line.rs`, `crates/core/src/view/scatter.rs`: validate `ViewRequest` (finite, min < max, width/height 1..=8192, else `INVALID_VIEW`); per Y series select points with x in range; ≤ 20,000 visible → raw in file order; line style: per-pixel-column first/min/max/last in order (monotonic X) or per-column min/max envelope (non-monotonic X), NaN breaks preserved; scatter: at most one point per occupied pixel cell; payload `(column_index, mode, interleaved [x, y] f64)`
- [ ] T028 [US1] Implement `Session::view(request)` (`NO_PLOT` when no plot) and binary encoding per `contracts/tauri-commands.md` in `crates/core/src/view/encode.rs`

### Shell commands

- [ ] T029 [US1] Implement Tauri command adapters in `src-tauri/src/commands/mod.rs`, `src-tauri/src/commands/data.rs`: `load_file`, `get_state`, `set_plot`, `get_view` (returns `tauri::ipc::Response` with the binary payload), `remove_recent_file`; register with the builder; every error returned as `ErrorReport`

### Kept tests (acceptance-critical)

- [ ] T030 [P] [US1] Write `crates/core/tests/import_valid.rs` over the T019 fixtures: header detection (incl. date-time first column), delimiters, quoting with line breaks (reported line = first line), blank lines and CRLF, missing tokens (all case variants → missing, not bad), column kinds, duplicate/empty names, offset handling, single column
- [ ] T031 [P] [US1] Write `crates/core/tests/view_faithfulness.rs`: a single-sample spike in 1M points appears in the reduced payload at full zoom-out and every zoom level; NaN gaps remain breaks; scatter occupancy inks every occupied pixel cell; raw mode below 20,000 visible points

### Frontend: data panel

- [ ] T032 [US1] Implement the data panel in `ui/src/data-panel/DataPanel.tsx`, `ui/src/data-panel/ColumnPicker.tsx`: File → Open via `@tauri-apps/plugin-dialog` (CSV/TSV/TXT filter, plus all files); shows file name, row count, and per column name, kind, missing count; X select limited to `usable_as_x`; Y multi-select limited to `usable_as_y` and excluding X; Line/Scatter toggle; calls `set_plot`; empty states per FR-005a; all controls keyboard-operable
- [ ] T033 [US1] Implement drag-and-drop and recent files in `ui/src/data-panel/drop.ts` and `ui/src/app/RecentFiles.tsx`: window drag-drop event loads a single dropped file; more than one → `MULTIPLE_FILES_DROPPED` notice, nothing loaded; File → Recent lists up to 10 entries from settings; `FILE_NOT_FOUND` from a recent entry → "file not found" with a Remove action calling `remove_recent_file`

### Frontend: plot

- [ ] T034 [US1] Implement the Pixi v8 renderer in `ui/src/plot/renderer/PlotCanvas.tsx`, `ui/src/plot/renderer/lineMesh.ts`, `ui/src/plot/renderer/markerMesh.ts`, `ui/src/plot/renderer/shaders.ts`: one `Application` (WebGL2, antialias, resize to container, DPR-aware); lines as instanced segment quads with constant pixel width; scatter as instanced quads with circular SDF (hollow variant); data→screen transform as a uniform so pan/zoom never rebuilds geometry; dashed variant for lines beyond the palette size; handle WebGL context loss by rebuilding
- [ ] T035 [US1] Implement view data flow in `ui/src/plot/viewData.ts`: request `get_view` when the view settles (50 ms debounce) and at most every 100 ms during continuous motion; decode the binary payload per `contracts/tauri-commands.md`; upload to meshes; discard out-of-date responses
- [ ] T036 [P] [US1] Implement axes in `ui/src/plot/axes/ticks.ts`, `ui/src/plot/axes/format.ts`, `ui/src/plot/axes/Axes.tsx`: `d3-scale` linear/time scales, ~1 tick per 80 px at nice values; numeric labels switch to scientific notation when |v| ≥ 1e6 or < 1e-3 (non-zero); time labels adapt sub-second → multi-year with date context at day boundaries (`d3-time-format`); gridlines; axis titles from column names; DOM overlay labels themed via CSS variables
- [ ] T037 [US1] Implement gestures and keyboard in `ui/src/plot/interaction/gestures.ts`, `ui/src/plot/interaction/keyboard.ts`: plain drag draws a box and zooms to it on release (drag < 5 px = click); Shift+drag or middle-drag pans; wheel zooms 1.2× per notch at the pointer; double-click resets; with focus, arrows pan, `+`/`-` zoom about center, `0` resets; clamp zoom between ~1e-6 × and 100 × full data range per axis
- [ ] T038 [US1] Implement view policy in `ui/src/plot/viewState.ts`: initial fit to visible series with 5% padding; on Y add/remove or style toggle keep X range and refit Y to visible data in that range; on X change reset; on window resize keep both ranges; "Reset view" button fits visible series; a zero-width data range (single row or constant column) gets a sensible non-zero range around the value
- [ ] T039 [US1] Implement the legend and palette in `ui/src/plot/legend/Legend.tsx`, `ui/src/plot/legend/palette.ts`: ≥ 8-color color-vision-deficiency-safe palette with light and dark variants each ≥ 3:1 contrast against the plot background; colors stable per column as others are added/removed; beyond palette size repeat with dashed line / hollow marker; click toggles hidden (hidden stays selected, shown as hidden, excluded from reset); scrolls on overflow; long names truncated with full name on hover; "all series hidden" empty state; a selected series with no plottable values is noted as such in the legend
- [ ] T040 [US1] Implement hover readout in `ui/src/plot/hover.ts` and `ui/src/plot/HoverTooltip.tsx`: nearest visible, non-missing point within 8 px across visible series (screen distance) from the current payload; tooltip shows series name, X (full date-time for date/time X), Y; none beyond 8 px
- [ ] T041 [P] [US1] Write kept Vitest tests `ui/src/plot/axes/format.test.ts` (scientific-notation thresholds; time labels at sub-second, day, and multi-year spans) and `ui/src/plot/hover.test.ts` (8 px radius, hidden series excluded, NaN skipped)
- [ ] T042 [US1] Implement the status bar in `ui/src/app/StatusBar.tsx`: indeterminate progress indicator with file name while `loading`; last load duration; API address slot (filled in US3)

**Checkpoint**: User Story 1 is fully functional — valid files load and plot interactively

---

## Phase 4: User Story 2 — Understand and fix a malformed file (Priority: P2)

**Goal**: Bad cells load as NaN with a complete, paged warning list; structural problems fail with precise, actionable errors; nothing crashes and a failed load leaves prior state intact

**Independent Test**: Open each malformed fixture per quickstart.md Story 2 and compare to expected codes, lines, columns, and values

- [ ] T043 [P] [US2] Create malformed fixtures in `fixtures/csv/`: `bad_cells.csv` (line 1,204 of `temp` = `abc`, plus `#DIV/0!` elsewhere and a 100-char text value), `ragged.csv` (one short and one long record), `ragged_no_header.csv`, `empty.csv` (0 bytes), `whitespace_only.csv`, `bom_only.csv`, `header_only.csv`, `latin1.csv` (Latin-1 `é` bytes on a known line/column), `decimal_comma.csv`, `unterminated_quote.csv`, `mixed_offsets.csv`
- [ ] T044 [US2] Implement `BadCellStore` in `crates/core/src/dataset/bad_cells.rs`: compact entries `(line: u64, column: u32, text_offset: u32, text_len: u16)` plus a shared text arena; texts > 64 chars stored truncated with `…`; exact total and per-column counts; `page(offset, limit ≤ 1000)` in file order; wire into the reader (T024) so every bad cell is recorded
- [ ] T045 [US2] Implement structural failures in `crates/core/src/csv_import/mod.rs` and `crates/core/src/csv_import/errors.rs`: `FIELD_COUNT_MISMATCH` (vs header, or first record when headerless; `details.expected_fields`/`actual_fields`; record's first line), `INVALID_ENCODING` (line + column), `MALFORMED_QUOTING` (line), `EMPTY_FILE` (0 bytes, whitespace-only, BOM-only), `NO_DATA_ROWS`, `FILE_NOT_FOUND`, `FILE_UNREADABLE` (permission, lock, directory; `details.os_error`), `OUT_OF_MEMORY` (from failed `try_reserve`); first structural problem in file order wins
- [ ] T046 [US2] Implement `Session::bad_cells(dataset_id, offset, limit)` in `crates/core/src/session/bad_cells.rs` (limit 1..=1000; unknown/stale id → `UNKNOWN_DATASET`) and the `get_bad_cells` Tauri command in `src-tauri/src/commands/data.rs`; include first ≤ 100 bad cells as `bad_cells_preview` in `LoadResult`
- [ ] T047 [P] [US2] Write `crates/core/tests/import_malformed.rs`: each T043 fixture yields the expected outcome (code, line, column, value, details) or, for bad cells, a successful load with exact totals, per-column counts, 64-char truncation, and paging; a failed load after a successful one leaves the previous dataset and plot unchanged
- [ ] T048 [US2] Implement error display in `ui/src/app/ErrorDisplay.tsx`: shows `message`, location (line, column), offending value, and `hint`; branches on `code` (e.g., `FILE_NOT_FOUND` from a recent entry → Remove action); dismissible; keyboard accessible
- [ ] T049 [US2] Implement bad-cell warnings in `ui/src/data-panel/BadCells.tsx`: data-panel summary "N bad cells" with per-column counts that opens a paged list (100 per page: line, column, value) via `get_bad_cells`; available until another file loads

**Checkpoint**: User Stories 1 and 2 both work independently

---

## Phase 5: User Story 3 — Load and plot from a local script (Priority: P3)

**Goal**: Every load/plot operation available over localhost HTTP+JSON, reflected live in the open window, with identical errors and warnings

**Independent Test**: Run `fixtures/scripts/load_and_plot.py` against valid and malformed files with the app open (quickstart.md Story 3)

- [ ] T050 [US3] Implement API server and discovery in `crates/api/src/lib.rs` and `crates/api/src/discovery/mod.rs`: `start(session, port) -> ApiHandle` binding `127.0.0.1:port`, falling back to `127.0.0.1:0` if busy; write `%LOCALAPPDATA%\plot-twist\api.json` (`base_url`, `pid`, `version`) atomically; remove on shutdown; never bind non-loopback addresses
- [ ] T051 [US3] Implement `/v1` routes in `crates/api/src/routes/mod.rs`, `crates/api/src/routes/data.rs`, `crates/api/src/routes/errors.rs` per `contracts/local-api.md`: `GET /health`, `POST /load`, `GET /state`, `PUT /plot`, `GET /datasets/{id}/bad-cells`, `GET /datasets/{id}/view` (JSON points with `null` breaks); all calls use `origin: Api`; map `PtError` → status (400/404/409/422/500) with `ErrorReport` body; JSON rejections → `BAD_REQUEST`; unexpected faults → `INTERNAL` with `details.log_id` logged
- [ ] T052 [US3] Start the API from the shell in `src-tauri/src/startup/api.rs` using `Settings.api_port`; emit `api-status` (`{ base_url }` or `{ error }`); show address or failure in `ui/src/app/StatusBar.tsx`
- [ ] T053 [US3] Implement API-origin notices in `ui/src/app/ApiNotices.tsx`: on `session-changed` with `origin: "api"` show a brief notice ("Loaded by script: run1.csv", "Plot updated by script"), show API-triggered errors via `ErrorDisplay` and warnings via `BadCells`; never request window focus; plot view policy (T038) applies identically to API plot changes
- [ ] T054 [P] [US3] Write `crates/api/tests/acceptance.rs`: start the API over an in-process `Session` on port 0 and replay Stories 1–3 over HTTP — load valid file (columns, row count, dataset id), `PUT /plot` then `GET /state` round-trip, `ragged.csv` → 422 `FIELD_COUNT_MISMATCH` with line and details, `bad_cells.csv` → 200 with preview and paging, unknown column → 404 `UNKNOWN_COLUMN`, stale dataset → 409, second concurrent load → 409 `LOAD_IN_PROGRESS`, view JSON contains the spike, server bound only to 127.0.0.1, `api.json` written and removed
- [ ] T055 [P] [US3] Write `fixtures/scripts/load_and_plot.py` (standard library only; reads `api.json`; loads the given file, plots given X and Y columns; prints dataset id and columns, or the `ErrorReport`) and user docs `docs/local-api.md` (discovery, endpoints, error codes, the example) derived from the contracts

**Checkpoint**: All user stories independently functional

---

## Phase 6: Polish & Cross-Cutting Concerns

- [ ] T056 Implement the release-mode perf test `crates/core/tests/perf.rs` (`#[ignore]`): generate a 1M×10 CSV in a temp dir; assert load < 3 s (SC-001), view reduction for 10 series < 50 ms, and process memory after load < 3× file size (SC-004, `memory-stats`); print measurements
- [ ] T057 Implement the startup check: `PLOT_TWIST_EXIT_WHEN_READY=1` in `src-tauri/src/startup/mod.rs` exits after the window's first frame and API readiness; `scripts/check-startup.ps1` builds release (`--no-bundle`), launches, polls `/v1/health`, and fails if > 2 s (SC-003); wire into `.github/workflows/ci.yml`
- [ ] T058 [P] Implement the FPS overlay toggle (Settings → Diagnostics) in `ui/src/plot/FpsOverlay.tsx` for manual 60 fps measurement (SC-002)
- [ ] T059 [P] Update `README.md` Development section with `npm ci`, `npm run tauri dev`, `npm run check`, and a link to `docs/local-api.md`
- [ ] T060 Audit code organization (constitution VII): no file > ~500 lines (excluding inline test modules), no function > ~100 lines, folder depth ≤ 3 below each source root; split where needed
- [ ] T061 Run all quality gates locally (`cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, `npm run check`); fix every failure; purge throwaway tests
- [ ] T062 Run `cargo test -p plot-twist-core --release --test perf -- --ignored --nocapture` and record results in the PR description
- [ ] T063 Run quickstart.md end to end (manual scenarios, Story 3 script, performance section) and record results, including measured FPS, in the PR description

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: none. T001 before T002–T004; T005 before T006.
- **Foundational (Phase 2)**: depends on Setup; blocks all stories. T010 → T011 → T013; T009 before T013; T013 → T014, T015; T015 → T016; T014 → T017 → T018.
- **US1 (Phase 3)**: depends on Foundational. T021–T023 → T024 → T025; T026 after T013; T027 → T028 → T029; T029 → T032–T040; T034 → T035, T037, T038, T039, T040.
- **US2 (Phase 4)**: depends on US1's reader (T024) and session load (T025). T044 → T045 → T046 → T049; T048 after T017.
- **US3 (Phase 5)**: depends on Foundational and the Session operations from US1 (T025, T026, T028) and US2 (T046 for bad-cells endpoint). T050 → T051 → T052 → T053.
- **Polish (Phase 6)**: after the desired stories.

### User Story Dependencies

- **US1 (P1)**: independent after Foundational.
- **US2 (P2)**: extends the US1 reader; independently testable with malformed fixtures.
- **US3 (P3)**: adapters over US1/US2 session operations; independently testable via HTTP.

### Parallel Opportunities

- Setup: T002, T003 (after T001); T006, T007, T008.
- Foundational: T009, T010, T012, T017 (after bindings exist).
- US1: T019, T020, T021, T022, T023, T027 in parallel; T030, T031, T036, T041 in parallel.
- US2: T043 and T047 alongside T048.
- US3: T054, T055 in parallel once routes exist.

---

## Parallel Example: User Story 1

```bash
Task: "Implement delimiter sniffing in crates/core/src/csv_import/sniff.rs"
Task: "Implement cell classification in crates/core/src/csv_import/cells.rs"
Task: "Implement header handling in crates/core/src/csv_import/header.rs"
Task: "Implement view reduction in crates/core/src/view/"
Task: "Create valid fixtures in fixtures/csv/"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Phase 1 Setup → Phase 2 Foundational
2. Phase 3 (US1) → validate with quickstart.md Story 1 and the perf test
3. Stop and demo

### Incremental Delivery

1. Foundation → US1 (MVP) → US2 (messy data) → US3 (scripting) → Polish
2. Each story is validated by its kept tests and quickstart section before moving on

---

## Notes

- [P] tasks = different files, no dependencies on incomplete tasks
- Keep only acceptance-critical tests; delete throwaway tests before merge (Principle III)
- No `unwrap`/`expect`/`panic!` outside tests (Principle V); generated TS types only (Principle II)
- Commit after each task or logical group
