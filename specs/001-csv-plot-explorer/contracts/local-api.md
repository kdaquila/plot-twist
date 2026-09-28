# Contract: Local HTTP API v1

Plain HTTP + JSON, bound to `127.0.0.1` only. Base URL `http://127.0.0.1:<port>/v1`.

## Discovery

- Default port **47811** (configurable in settings). If busy, the app binds an
  OS-assigned port.
- The actual address is always written to `%LOCALAPPDATA%\plot-twist\api.json`:

```json
{ "base_url": "http://127.0.0.1:47811/v1", "pid": 12345, "version": "0.1.0" }
```

- The file is removed on clean exit. A stale file (pid not running) means the app is not
  running.

## Conventions

- Request/response bodies are `application/json; charset=utf-8`.
- Success: `200`. Client error: `400` (bad request shape), `404` (unknown dataset or
  column), `409` (stale dataset / load in progress), `422` (load or plot failure described
  by the file or request contents). Server fault: `500`.
- Requests whose `Host` header is not `127.0.0.1:<port>` or `localhost:<port>` are refused
  with `403` (`BAD_REQUEST`), so web pages cannot reach the API through DNS rebinding.
  Unknown paths return `404` (`BAD_REQUEST`).
- Every non-2xx body is an **ErrorReport**:

```json
{
  "code": "FIELD_COUNT_MISMATCH",
  "message": "Line 1,204 has 7 fields, but the header has 8.",
  "hint": "Add or remove fields on that line so it matches the header, then reopen the file.",
  "line": 1204,
  "column": null,
  "value": null,
  "details": { "expected_fields": 8, "actual_fields": 7 }
}
```

Codes: see [error-codes.md](error-codes.md).

## Endpoints

### `GET /v1/health`

`200 {"app": "plot-twist", "version": "0.1.0", "api_version": 1}`

### `POST /v1/load`

Request: `{"path": "C:\\data\\run1.csv"}` (absolute path).

Response `200` — **LoadResult**:

```json
{
  "dataset": {
    "id": "ds-3",
    "path": "C:\\data\\run1.csv",
    "file_name": "run1.csv",
    "row_count": 1000000,
    "has_header": true,
    "delimiter": "comma",
    "columns": [
      {"index": 0, "name": "time", "kind": "datetime", "missing_count": 0,
       "bad_cell_count": 0, "usable_as_x": true, "usable_as_y": false,
       "min": 1790000000.0, "max": 1790086399.0},
      {"index": 1, "name": "temp", "kind": "numeric", "missing_count": 3,
       "bad_cell_count": 1, "usable_as_x": true, "usable_as_y": true,
       "min": 12.5, "max": 30.1}
    ],
    "bad_cell_total": 1
  },
  "bad_cells_preview": [
    {"line": 1204, "column": 1, "column_name": "temp", "text": "abc"}
  ]
}
```

`bad_cells_preview` holds the first ≤ 100 bad cells; page the rest with
`GET /v1/datasets/{id}/bad-cells`. The open window shows the loaded file and warnings.
Failure → `422` ErrorReport; the window shows the same error; previous dataset unchanged.
A load already in progress → `409 LOAD_IN_PROGRESS`.

### `GET /v1/state`

`200 {"dataset": <DatasetSummary|null>, "plot": <PlotConfig|null>, "loading": {"path": "..."}|null}`

### `PUT /v1/plot`

Request — **PlotConfig**:

```json
{"dataset_id": "ds-3", "x": "time", "y": ["temp", "pressure"], "style": "line"}
```

`200` echoes the stored PlotConfig; the window redraws. If X is unchanged, it keeps the X
range and refits Y; if X changed, it resets the view to fit (spec FR-009d2).
Failures: `404 UNKNOWN_COLUMN`, `409 STALE_DATASET`, `422 COLUMN_NOT_USABLE_AS_X |
COLUMN_NOT_USABLE_AS_Y | X_ALSO_Y | NO_Y_COLUMNS | DUPLICATE_Y_COLUMN`. Current plot
unchanged on failure.

### `GET /v1/datasets/{id}/bad-cells?offset=0&limit=100`

`limit` 1..=1000. `200 {"total": 12, "offset": 0, "items": [BadCell, ...]}`.
Unknown/stale id → `404 UNKNOWN_DATASET`.

### `GET /v1/datasets/{id}/view?x_min=&x_max=&y_min=&y_max=&width=&height=`

Reduced render data for the current plot's series over the given range (same reduction the
GUI uses). `200`:

```json
{"series": [{"column": "temp", "mode": "reduced", "y_min": 12.5, "y_max": 30.1,
             "points": [[x, y], [x, y], null, ...]}]}
```

`null` marks a line break (missing value). Requires a current plot → otherwise
`409 NO_PLOT`.

## Versioning

Additive changes (new fields, endpoints) keep `/v1`. Removing or changing a field's meaning
requires `/v2`. Clients must ignore unknown fields.

## Minimal Python example (standard library only)

```python
import json, os, urllib.request

info = json.load(open(os.path.expandvars(r"%LOCALAPPDATA%\plot-twist\api.json")))
base = info["base_url"]

def call(method, path, body=None):
    req = urllib.request.Request(base + path, method=method,
                                 data=json.dumps(body).encode() if body else None,
                                 headers={"Content-Type": "application/json"})
    with urllib.request.urlopen(req) as r:
        return json.load(r)

ds = call("POST", "/load", {"path": r"C:\data\run1.csv"})["dataset"]
call("PUT", "/plot", {"dataset_id": ds["id"], "x": "time",
                      "y": ["temp"], "style": "line"})
```
