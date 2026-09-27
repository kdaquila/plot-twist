# plot-twist local API

While plot-twist is running, scripts on the same computer can load files and set the plot
through a small HTTP + JSON API. Everything a script does appears in the open window
immediately, and everything done in the window is visible to the script.

The API accepts connections from this computer only (it listens on `127.0.0.1`).

## Finding the API

The app writes its address to `%LOCALAPPDATA%\plot-twist\api.json` when it starts:

```json
{ "base_url": "http://127.0.0.1:47811/v1", "pid": 12345, "version": "0.1.0" }
```

The default port is 47811 (`api_port` in `%APPDATA%\plot-twist\settings.json`). If that port
is busy, the app picks a free one, so always read `base_url` from the file. The file is
removed when the app exits. The status bar shows the address too.

## Quick start (Python, standard library only)

```python
import json, os, urllib.request

base = json.load(open(os.path.expandvars(r"%LOCALAPPDATA%\plot-twist\api.json")))["base_url"]

def call(method, path, body=None):
    req = urllib.request.Request(base + path, method=method,
                                 data=json.dumps(body).encode() if body else None,
                                 headers={"Content-Type": "application/json"})
    with urllib.request.urlopen(req) as r:
        return json.load(r)

ds = call("POST", "/load", {"path": r"C:\data\run1.csv"})["dataset"]
call("PUT", "/plot", {"dataset_id": ds["id"], "x": "time", "y": ["temp"], "style": "line"})
```

A complete example that prints errors nicely is in
[`fixtures/scripts/load_and_plot.py`](../fixtures/scripts/load_and_plot.py):

```
python fixtures/scripts/load_and_plot.py fixtures/csv/sensors.csv time temp pressure
```

## Endpoints

All paths are relative to `base_url`.

| Method | Path | Does |
|--------|------|------|
| `GET` | `/health` | `{"app": "plot-twist", "version": "0.1.0", "api_version": 1}` |
| `POST` | `/load` | Body `{"path": "<absolute path>"}`. Loads the file, replacing the current one on success. Returns the dataset summary and the first 100 bad cells. |
| `GET` | `/state` | `{"dataset": …, "plot": …, "loading": …}` |
| `PUT` | `/plot` | Body `{"dataset_id": "ds-1", "x": "time", "y": ["temp"], "style": "line" \| "scatter"}`. Sets the plot; returns it. |
| `GET` | `/datasets/{id}/bad-cells?offset=0&limit=100` | A page of bad cells (`limit` 1–1000). |
| `GET` | `/datasets/{id}/view?x_min=&x_max=&y_min=&y_max=&width=&height=` | The plotted points for that range at that pixel size, reduced the same way the window draws them. `null` in `points` marks a gap. |

Each successful load gets a new dataset id (`"ds-1"`, `"ds-2"`, …). Pass the current id
when setting the plot; an older id is refused with `STALE_DATASET`.

Columns are referenced by name. X can be a numeric or date/time column; Y columns must be
numeric. Date/time values are seconds since 1970-01-01 (UTC for values with an offset, as
written otherwise).

## Errors

Every failure returns an HTTP error status and the same report the window shows:

```json
{
  "code": "FIELD_COUNT_MISMATCH",
  "message": "Line 3 has 2 fields, but the header has 3.",
  "hint": "Add or remove fields on that line so it matches the header, then reopen the file.",
  "line": 3,
  "column": null,
  "value": null,
  "details": { "expected_fields": 3, "actual_fields": 2 }
}
```

| Status | Meaning |
|--------|---------|
| 400 | The request is malformed (`BAD_REQUEST`, `INVALID_VIEW`) |
| 403 | The request did not come from this computer |
| 404 | Unknown dataset or column (`UNKNOWN_DATASET`, `UNKNOWN_COLUMN`) |
| 409 | Out of date or busy (`STALE_DATASET`, `LOAD_IN_PROGRESS`, `NO_PLOT`) |
| 422 | The file or plot request can't be used (load failures, `COLUMN_NOT_USABLE_AS_X`, `COLUMN_NOT_USABLE_AS_Y`, `X_ALSO_Y`, `NO_Y_COLUMNS`, `DUPLICATE_Y_COLUMN`) |
| 500 | Internal fault (`INTERNAL`, with a `log_id` to include in a bug report) |

A failed load or plot change leaves the current file and plot unchanged. Cells that don't
match their column's type don't fail a load: they become missing values and are listed as
bad cells (code `BAD_CELL`).

The full list of codes and hints is in
[`specs/001-csv-plot-explorer/contracts/error-codes.md`](../specs/001-csv-plot-explorer/contracts/error-codes.md).

## Compatibility

New fields and endpoints may be added under `/v1`; ignore fields you don't recognize.
Breaking changes will use a new `/v2` prefix.
