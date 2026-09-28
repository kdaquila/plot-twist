# Contract: Error and Warning Codes

The closed vocabulary required by constitution Principle V and FR-012. Codes are stable
public API: never renamed or reused; new codes may be added. Messages and hints are
authored in Rust (`PtError`) and shown identically by the GUI and returned by the API.

## Load failures (dataset unchanged)

| Code | When | Fields | HTTP |
|------|------|--------|------|
| `FILE_NOT_FOUND` | Path does not exist | `details.path` | 422 |
| `FILE_UNREADABLE` | Permission denied, locked, is a directory, other I/O error | `details.path`, `details.os_error` | 422 |
| `EMPTY_FILE` | No bytes, or only whitespace | — | 422 |
| `NO_DATA_ROWS` | Header present, zero data rows | — | 422 |
| `FIELD_COUNT_MISMATCH` | A record's field count ≠ header/first-record count | `line`, `details.expected_fields`, `details.actual_fields` | 422 |
| `INVALID_ENCODING` | Bytes are not valid UTF-8 | `line`, `column` | 422 |
| `MALFORMED_QUOTING` | Unterminated quote or stray quote the parser cannot recover from | `line` | 422 |
| `OUT_OF_MEMORY` | Allocation for column storage failed | `details.file_size_bytes` | 422 |
| `LOAD_IN_PROGRESS` | Another load is running | — | 409 |

## Plot failures (plot unchanged)

| Code | When | Fields | HTTP |
|------|------|--------|------|
| `UNKNOWN_DATASET` | Dataset id never existed | `details.dataset_id` | 404 |
| `STALE_DATASET` | Dataset id is not the currently loaded one | `details.dataset_id`, `details.current_dataset_id` | 409 |
| `UNKNOWN_COLUMN` | Column name not in dataset | `column` | 404 |
| `COLUMN_NOT_USABLE_AS_X` | Text column chosen as X | `column` | 422 |
| `COLUMN_NOT_USABLE_AS_Y` | Text or date/time column chosen as Y | `column` | 422 |
| `X_ALSO_Y` | X column also listed in Y | `column` | 422 |
| `NO_Y_COLUMNS` | Y list empty | — | 422 |
| `DUPLICATE_Y_COLUMN` | Same column listed twice in Y | `column` | 422 |
| `NO_PLOT` | View requested with no current plot | — | 409 |
| `INVALID_VIEW` | Non-finite or inverted range, size out of 1..=8192 | `details` | 400 |

## Request failures (API only)

| Code | When | HTTP |
|------|------|------|
| `BAD_REQUEST` | Malformed JSON or missing/invalid fields | 400 |
| `INTERNAL` | Unexpected fault; logged with a correlation id in `details.log_id` | 500 |

## Warnings (load succeeds)

| Code | When | Fields |
|------|------|--------|
| `BAD_CELL` | A cell doesn't match its column's type; stored as missing (NaN) | `line`, `column`, `value` |

## Suggested next steps (hints)

| Code | Hint |
|------|------|
| `FILE_NOT_FOUND` | Check the path, or pick the file again with File → Open. |
| `FILE_UNREADABLE` | Close any program that has the file open and check you have permission to read it. |
| `EMPTY_FILE` | Choose a file that contains data. |
| `NO_DATA_ROWS` | Add at least one data row below the header. |
| `FIELD_COUNT_MISMATCH` | Add or remove fields on that line so it matches the header, then reopen the file. |
| `INVALID_ENCODING` | Re-save the file as UTF-8 (in Excel: "CSV UTF-8"). |
| `MALFORMED_QUOTING` | Close or remove the unmatched quote on that line. |
| `OUT_OF_MEMORY` | Close other programs or split the file into smaller files. |
| `LOAD_IN_PROGRESS` | Wait for the current load to finish, then try again. |
| `UNKNOWN_DATASET` / `STALE_DATASET` | Get the current dataset id from `GET /v1/state` and retry. |
| `UNKNOWN_COLUMN` | Use one of the column names listed for the loaded file. |
| `COLUMN_NOT_USABLE_AS_X` | Choose a numeric or date/time column for X. |
| `COLUMN_NOT_USABLE_AS_Y` | Choose numeric columns for Y. |
| `X_ALSO_Y` | Remove the X column from the Y list. |
| `NO_Y_COLUMNS` | Choose at least one Y column. |
| `DUPLICATE_Y_COLUMN` | List each Y column only once. |
| `NO_PLOT` | Set a plot with `PUT /v1/plot` first. |
| `INVALID_VIEW` | Use finite ranges with min < max and sizes between 1 and 8192. |
| `BAD_REQUEST` | Check the request body against the API documentation. |
| `INTERNAL` | Please report this issue and include the log id. |
| `BAD_CELL` | Fix or remove the value on that line if it should be a number or date. |

## GUI-only notices

| Code | When |
|------|------|
| `MULTIPLE_FILES_DROPPED` | More than one file dropped on the window (FR-001a) |
| `RECENT_FILE_MISSING` | A recent-files entry no longer exists; GUI offers removal (FR-001b) — surfaced from `FILE_NOT_FOUND` when the load originated from the recent list |
