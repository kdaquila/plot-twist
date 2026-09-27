# Contract: Tauri Commands and Events (GUI ↔ backend)

Every command is a thin adapter over the same `Session` operation the HTTP API uses
(constitution Principle I). Argument and result types are generated TypeScript (`ts-rs`)
in `ui/src/backend/generated/`. Failures reject with an `ErrorReport`
([error-codes.md](error-codes.md)).

| Command | Args | Result | HTTP equivalent |
|---------|------|--------|-----------------|
| `load_file` | `{ path: string }` | `LoadResult` | `POST /v1/load` |
| `get_state` | — | `SessionState` | `GET /v1/state` |
| `set_plot` | `PlotConfig` | `PlotConfig` | `PUT /v1/plot` |
| `get_bad_cells` | `{ datasetId, offset, limit }` | `BadCellPage` | `GET /v1/datasets/{id}/bad-cells` |
| `get_view` | `ViewRequest` | binary `ViewPayload` | `GET /v1/datasets/{id}/view` (JSON) |
| `get_settings` | — | `Settings` | — (app preference, not a data op) |
| `set_theme` | `{ theme }` | `Settings` | — |
| `remove_recent_file` | `{ path }` | `Settings` | — |

## Events (backend → GUI)

| Event | Payload | Purpose |
|-------|---------|---------|
| `session-changed` | `SessionEvent` | Any state change from any client; GUI refreshes and, for `origin: "api"`, shows API-triggered errors/warnings (FR-015, FR-016). |
| `api-status` | `{ base_url } \| { error: ErrorReport }` | Shown in the status bar so users can see where the API is listening. |

## Binary ViewPayload layout (little-endian)

```text
u32 series_count
repeat series_count:
  u32 column_index
  u8  mode            (0 = raw, 1 = reduced)
  u32 point_count
  f64 x[point_count]  interleaved with
  f64 y[point_count]  as [x0, y0, x1, y1, ...]; NaN y = line break
```
