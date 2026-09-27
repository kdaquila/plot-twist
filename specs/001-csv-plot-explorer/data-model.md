# Data Model: CSV Plot Explorer

**Feature**: [spec.md](spec.md) | **Research**: [research.md](research.md)

All entities live in `plot-twist-core`. Types crossing the Tauri boundary derive `ts-rs`
TypeScript definitions; the same shapes are serialized as JSON on the local API.

## Identifiers (newtypes)

| Type | Wraps | Notes |
|------|-------|-------|
| `DatasetId` | `u64` | Monotonic per app run; new on every successful load (FR-014a). Serialized as a string (`"ds-7"`) so scripts treat it as opaque. |
| `ColumnIndex` | `u32` | 0-based position in the file. |
| `LineNumber` | `u64` | 1-based line in the file as seen in a text editor; header is line 1 (spec Edge Cases). For records spanning lines (quoted newlines), the record's first line. |

## Dataset

| Field | Type | Rules |
|-------|------|-------|
| `id` | `DatasetId` | |
| `path` | absolute path | As loaded. |
| `file_name` | string | Display name. |
| `file_size_bytes` | u64 | Used by the memory budget check. |
| `row_count` | u64 | Data rows (excludes header). ≥ 1 — zero rows is a load error. |
| `has_header` | bool | False when the first record is all-numeric. |
| `delimiter` | `Comma \| Semicolon \| Tab` | Detected. |
| `columns` | `Vec<Column>` | ≥ 1, in file order. |
| `bad_cells` | `BadCellStore` | See below; not serialized whole — paged. |

Lifecycle: created only by a successful load; replaced atomically by the next successful
load; never mutated.

## Column

| Field | Type | Rules |
|-------|------|-------|
| `index` | `ColumnIndex` | |
| `name` | string | Unique within the dataset: header text, disambiguated as `name (2)`, `name (3)` … for duplicates; `Column N` without header. Empty header → `Column N`. |
| `kind` | `Numeric \| DateTime \| Text` | Set by the first non-empty, non-missing-token value (spec Edge Cases). A column with no such value is `Numeric` (all missing). |
| `missing_count` | u64 | Empty cells + missing tokens + non-finite + bad cells. |
| `bad_cell_count` | u64 | Subset of `missing_count`. |
| `usable_as_x` | bool | `kind ∈ {Numeric, DateTime}`. |
| `usable_as_y` | bool | `kind == Numeric`. |
| values | `Vec<f64>` (Numeric/DateTime only) | NaN = missing. DateTime = seconds since 1970-01-01 (naive timeline). Text columns store no values. |

## BadCell / BadCellStore

| Field | Type | Rules |
|-------|------|-------|
| `line` | `LineNumber` | |
| `column` | `ColumnIndex` | |
| `column_name` | string | Filled when serialized. |
| `text` | string | Original cell text; > 64 chars truncated with `…`. |

Store: compact entries + shared text arena; exact `total`; paged access
`(offset, limit ≤ 1000)` in file order.

## PlotConfig

| Field | Type | Rules |
|-------|------|-------|
| `dataset_id` | `DatasetId` | Must equal the loaded dataset's id, else `STALE_DATASET`. |
| `x` | column name | Must exist (`UNKNOWN_COLUMN`) and be `usable_as_x` (`COLUMN_NOT_USABLE_AS_X`). |
| `y` | `Vec<column name>` | 1..=N, unique, each exists and `usable_as_y`; must not contain `x` (`X_ALSO_Y`); empty → `NO_Y_COLUMNS`. |
| `style` | `Line \| Scatter` | |

Columns are referenced by their unique display name in both the Tauri commands and the
API. Series visibility (legend toggle) and the current zoom are view state owned by the
frontend, not part of `PlotConfig` (constitution Principle I exempts pure view concerns).

## SessionState

| Field | Type |
|-------|------|
| `dataset` | `Option<DatasetSummary>` (Dataset minus values and bad-cell entries; includes `bad_cell_total`) |
| `plot` | `Option<PlotConfig>` |
| `loading` | `Option<LoadingInfo { path }>` |

Transitions:

```text
Empty ──load ok──▶ Loaded(dataset, plot=None)
Loaded ──set_plot ok──▶ Loaded(dataset, plot=Some)
Loaded ──load ok──▶ Loaded(new dataset, plot=None)
any ──load fails / set_plot fails──▶ unchanged (error reported)
```

## SessionEvent (backend → GUI)

`DatasetLoaded { summary, warnings_total }` · `LoadFailed { report }` · `PlotChanged
{ plot }` · `LoadStarted { path }`. Each carries an `origin: Gui | Api` so the GUI can
surface API-originated errors/warnings (FR-016).

## ErrorReport

| Field | Type | Notes |
|-------|------|-------|
| `code` | string | Stable code; see [contracts/error-codes.md](contracts/error-codes.md). |
| `message` | string | What went wrong. |
| `hint` | string | Suggested next step. |
| `line` | `Option<LineNumber>` | |
| `column` | `Option<string>` | Column name (or `Column N`). |
| `value` | `Option<string>` | Offending text, truncated to 64 chars. |
| `details` | `Option<object>` | Code-specific (e.g., `expected_fields`, `actual_fields`). |

## ViewRequest / ViewPayload

Request: `dataset_id`, `x_min`, `x_max`, `y_min`, `y_max` (f64, same units as stored
values), `width_px`, `height_px` (u32, 1..=8192), plus the current `PlotConfig` series.

Payload per Y series: `column`, `mode: Raw | Reduced`, and interleaved `[x, y]` f64 pairs
in draw order (NaN pair = line break). Tauri returns it as a binary buffer
(header + f64 LE arrays); the HTTP API returns JSON arrays.

## Settings

`theme: System | Light | Dark` (default System) · `api_port: u16` (default 47811) ·
`recent_files: Vec<path>` (≤ 10, most recent first, de-duplicated, updated on every
successful load from any client).
