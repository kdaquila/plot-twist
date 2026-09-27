# Feature Specification: CSV Plot Explorer

**Feature Branch**: `001-csv-plot-explorer`

**Created**: 2026-09-27

**Status**: Draft

**Input**: User description: "Load a CSV file and explore it as an interactive plot: the user opens a numeric CSV, picks an X column and one or more Y columns, and gets a pannable/zoomable line or scatter plot. Loading and plotting are also available through the local API, with results appearing in the open GUI. Malformed files produce clear errors naming the row and column."

## Clarifications

### Session 2026-09-27

- Q: When a numeric column has a few bad cells, should the load fail or continue? → A: Continue. Bad cells are stored as NaN (missing), and the file loads with a warning listing each bad cell's row, column, and value. Structural problems still fail the load.
- Q: Should the first version support date/time columns as the X axis? → A: Yes, ISO 8601 only. Such columns are detected and can be used as X, shown on a time axis; other date formats are treated as text.
- Q: Should decimal-comma numbers (e.g., `1,5`) be read? → A: Not in this feature; only `.` is accepted as the decimal separator.
- Q: Which plot interactions beyond pan, scroll-zoom, and reset belong in this version? → A: Hover readout, box zoom, and legend click to hide/show a series. Single-axis zoom is out of scope.
- Q: How do mouse gestures map to box zoom and pan? → A: Plotly-like: plain drag = box zoom; Shift+drag (or middle-drag) = pan.
- Q: When the plotted columns or style change, is the zoom kept or reset? → A: Adding/removing Y columns or toggling style keeps the X range and refits Y to the visible data; changing the X column resets the view; window resize keeps both ranges.
- Q: Can an in-progress load be cancelled? → A: Not in this feature; loads run to completion.
- Q: Should first-time-user usability checks (former SC-005/SC-006) be formal criteria? → A: No; they were removed from the success criteria.
- Q: One loaded file at a time, or several? → A: One at a time; opening a new file replaces the current one. The local API identifies the loaded dataset by an ID so multiple datasets can be added later without breaking scripts.
- Q: How should scripts talk to the local API in v1? → A: A documented plain HTTP + JSON protocol with a short Python example that uses only the standard library; no client package is published in this feature.
- Q: Should v1 support logarithmic axes? → A: Not in this feature; axes are linear (or time for date/time X).
- Q: Besides the open dialog, how else can users open a file in v1? → A: Drag and drop onto the window, and a recent-files list. No command-line argument / "Open with" in this feature.
- Q: How should light/dark appearance work? → A: Follow the Windows app-mode setting by default (updating live), with an in-app Light / Dark / System choice that is remembered across restarts.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Open a CSV and explore it as a plot (Priority: P1)

A researcher has a CSV of numeric measurements (for example, time and several sensor
readings). They open the file in the app, see the list of its columns, choose one column
for the X axis and one or more columns for the Y axis, and choose line or scatter style.
The plot appears immediately. They drag a box to zoom into a region of interest, scroll to
zoom, Shift+drag to pan,
and reset the view to see everything again.

**Why this priority**: This is the core value of the product — going from a data file to
an explorable picture of it. Every later feature builds on it.

**Independent Test**: Open a sample numeric CSV, select X and two Y columns, confirm both
series are drawn with a legend, pan and zoom, and reset the view.

**Acceptance Scenarios**:

1. **Given** the app is open with no data, **When** the user opens a valid numeric CSV,
   **Then** the app shows the file name, row count, and the list of columns.
2. **Given** a loaded file, **When** the user selects an X column and one or more Y
   columns, **Then** a plot is drawn with one series per Y column, each in a distinct
   color, with a legend naming each series and axes labeled with the column names.
3. **Given** a plot is shown, **When** the user switches between line and scatter style,
   **Then** all series are redrawn in the chosen style without reloading the file.
4. **Given** a plot is shown, **When** the user Shift+drags on the plot, **Then** the view pans;
   **When** the user scrolls on the plot, **Then** the view zooms in or out centered on the
   pointer; **When** the user drags on the plot, **Then** a zoom rectangle is drawn and the view zooms to that
   region; **When** the user resets the view, **Then** all visible series fit in view.
5. **Given** a plot is shown, **When** the user hovers near a point, **Then** the series
   name and the point's exact X and Y values are shown.
6. **Given** a plot with several series, **When** the user clicks a series' legend entry,
   **Then** that series is hidden (or shown again) without changing the selection.
7. **Given** a plot is shown, **When** the user adds or removes a Y column, **Then** the
   plot updates to show exactly the selected series.
8. **Given** a file is loaded, **When** the user opens a different file, **Then** the new
   file replaces the previous one and the plot selection is cleared.
9. **Given** a file whose first column holds ISO 8601 date-times, **When** the user selects
   it as X, **Then** the X axis shows readable dates/times that adapt as the user zooms.

---

### User Story 2 - Understand and fix a malformed file (Priority: P2)

A researcher opens a file that has a problem. Small cell-level problems — a stray word
or spreadsheet error code in a numeric column — don't block them: the file loads, the bad
cells become missing values, and the app lists exactly which cells were affected. Problems
with the file's structure — a row with the wrong number of fields, unreadable text, or no
data at all — stop the load with a message saying exactly what is wrong, where in the file
it is, and what to do. Either way, there is no crash and no silent wrong plot, and
whatever was loaded before a failed load remains intact.

**Why this priority**: Real research data is messy. A tool that fails vaguely or plots
wrong data silently loses the user's trust immediately. This is second only to plotting
itself.

**Independent Test**: Open a set of deliberately malformed files and confirm each produces
a message naming the problem, the row, and (where applicable) the column, and that the app
continues to work afterward.

**Acceptance Scenarios**:

1. **Given** a CSV where row 1,204 of numeric column `temp` contains `abc`, **When** the
   user opens it, **Then** the file loads, that cell is treated as missing (NaN), and a
   warning shows the number of bad cells and lists row 1,204, column `temp`, value `abc`.
2. **Given** a CSV where one row has fewer or more fields than the header (or the first
   row, if there is no header), **When** the
   user opens it, **Then** the load fails with a message identifying that row and the
   expected versus actual number of fields.
3. **Given** an empty file, or a file with a header but no data rows, **When** the user
   opens it, **Then** the load fails with a message saying the file contains no data.
4. **Given** a file is already loaded and plotted, **When** the user tries to open a
   malformed file, **Then** the error is shown and the previously loaded data and plot
   remain unchanged.
5. **Given** any load error, **When** it is shown, **Then** it includes a suggested next
   step (for example, "fix or remove the value on that row and reopen the file").

---

### User Story 3 - Load and plot from a local script (Priority: P3)

A researcher runs a script on the same computer that tells the running app to load a file
and plot chosen columns. The file and plot appear in the open app window, where the
researcher can explore them interactively exactly as if they had done it by hand. The
script can also ask what file is loaded, what columns it has, and what is currently
plotted.

**Why this priority**: Automation by scripts (and later, AI agents) is a first-class way to
use the product. This story proves that every action in Stories 1 and 2 is reachable
without the GUI.

**Independent Test**: With the app open, run a script that loads a file and requests a
plot of specific columns; confirm the plot appears in the window, and that the script
receives the same structured error information for a malformed file that the GUI shows.

**Acceptance Scenarios**:

1. **Given** the app is running, **When** a local script requests that a file be loaded,
   **Then** the file loads in the open window and the script receives the file's columns
   and row count.
2. **Given** a file is loaded, **When** a local script requests a plot of an X column and
   one or more Y columns in line or scatter style, **Then** the open window shows that plot
   and the user can immediately pan and zoom it.
3. **Given** a script requests loading of a malformed file, **When** the load fails,
   **Then** the script receives a stable, machine-readable error code plus the row,
   column, and message, and the window shows the same error to the user. Likewise, a load
   with bad cells returns the same warnings the window shows.
4. **Given** the user has loaded and plotted a file by hand, **When** a script asks for the
   current state, **Then** it receives the loaded file, its columns, and the current plot
   selection and style.
5. **Given** a script requests a plot using a column name that does not exist, **When** the
   request is processed, **Then** the script receives an error naming the unknown column
   and the current plot is unchanged.

---

### Edge Cases

- **No header row**: If every field in the first row is a number or an ISO 8601 date/time,
  the file is treated as having no header; all rows are data and columns are named
  `Column 1`, `Column 2`, …. Field counts are then checked against the first row.
- **Blank lines and line endings**: CRLF and LF line endings are both accepted. Completely
  blank lines (anywhere, including at the end) are skipped, not counted as data, and do not
  cause field-count errors; they still count toward line numbers.
- **Empty file**: A file with no bytes, only whitespace, or only a byte-order mark is an
  empty file.
- **Number formats**: Accepted: optional leading `+` or `-`, decimals with `.`, and
  scientific notation (`1.5e-3`). Surrounding spaces are ignored. Thousands separators are
  not accepted (such values are bad cells).
- **Single column**: A file with no detectable delimiter is loaded as a single column.
  When the delimiter sample is ambiguous, comma is used.
- **Duplicate column names**: Columns with the same header name are distinguished by
  position (e.g., `temp` and `temp (2)`).
- **Column types**: Each column's type is set by its first value that is neither empty
  nor a missing-value token: a number
  makes it numeric; an ISO 8601 date or date-time (e.g., `2026-09-27`,
  `2026-09-27T14:03:22`, `2026-09-27 14:03:22.125`, optionally with `Z` or a `+02:00`
  offset) makes it a date/time column; anything else makes it a text column. Text columns
  are listed but marked unavailable for plotting and do not cause a load error. A later
  value that doesn't match its column's type keeps loading: that cell becomes missing
  (NaN) and is listed as a bad cell (Story 2). A column with no typed values at all is
  numeric (all missing).
- **Date/time columns**: They can be used as X only (not as Y). Values without an offset
  are plotted as written, with no time zone conversion; values with an offset are
  plotted in UTC. Date-only values mean midnight and may be mixed with date-times.
  Fractional seconds are kept to at least microsecond precision. If the column's first
  value has an offset, later values without one are bad cells, and vice versa. Non-ISO
  formats (e.g., `09/27/2026`) are text.
- **Missing values**: Empty cells and the tokens `NaN`, `NA`, `N/A`, `#N/A`, `null`, and
  `None` (any letter case) are missing values, not errors. Non-finite values (`inf`,
  `-inf`) are also treated as missing. The
  number of missing values per column is shown. Line plots show a gap at missing values;
  scatter plots omit those points.
- **Delimiters**: Comma, semicolon, and tab delimiters are detected automatically. Quoted
  fields (including quoted delimiters) are supported.
- **Decimal separator**: Only `.` is accepted as the decimal separator; a value such as
  `1,5` in a numeric column of a semicolon-delimited file is a bad cell (missing, and
  listed with its row and column). A column whose first value uses a decimal comma is a
  text column.
- **Encoding**: Files must be UTF-8 (with or without byte-order mark); invalid text
  encoding is reported with the row where it occurs.
- **Unsorted X values**: In line style, points are connected in file order, not re-sorted
  by X.
- **Single row or constant column**: The plot still renders with a sensible non-zero axis
  range around the value.
- **X column selected as Y**: The current X column cannot also be selected as a Y series.
- **Column with all values missing**: It can be selected but draws nothing; the legend
  notes it has no plottable values.
- **Row numbering**: Row numbers in messages are the line numbers of the file as seen in a
  text editor (1-based, header counted as line 1). For a record containing quoted line
  breaks, the record's first line is reported.
- **Locked or unreadable files**: A file that cannot be read (for example, locked by
  another program or access denied) fails the load with a message suggesting the user
  close the other program or check permissions.
- **Too large for memory**: If the file's data cannot fit in available memory, the load
  fails with a message saying so; the app keeps running with its previous data.
- **Concurrent loads**: While a load is in progress, another load request (from the
  window or a script) is refused with a "load already in progress" error.
- **Multiple problems in one file**: Every bad cell is listed as a warning. For structural
  problems, the first one found (in file order) is reported.
- **Bad X values**: A bad cell in the X column makes that row's point missing for every
  series.
- **Second launch**: Launching the app while it is already running brings the existing
  window to the front instead of starting a second copy, so scripts always reach the one
  open window.

## Requirements *(mandatory)*

### Functional Requirements

**Loading**

- **FR-001**: Users MUST be able to open a CSV file from their computer through a standard
  file-open dialog.
- **FR-001a**: Users MUST be able to open a file by dragging it onto the app window.
  Dropping more than one file at once MUST show a message asking for a single file and
  load nothing.
- **FR-001b**: The app MUST keep a list of the 10 most recently opened files, remembered
  across restarts, from which users can reopen a file. Choosing a file that no longer
  exists MUST show a "file not found" message and offer to remove it from the list.
- **FR-002**: The system MUST parse the file according to the rules in Edge Cases
  (header detection, delimiters, quoting, missing values, text columns, encoding).
- **FR-003**: After a successful load, the system MUST display the file name, number of
  data rows, and each column's name, type, and missing-value count.
- **FR-004**: The system MUST hold one loaded file at a time; loading a new file replaces
  the current one only after the new file loads successfully.
- **FR-005**: The system MUST show that a load is in progress (with a progress indicator
  in the status bar) and remain responsive while loading; the current plot, if any, stays
  interactive until the new file finishes loading.
- **FR-005a**: The window MUST show clear empty states: with no file loaded, a prompt to
  open or drop a file; with a file but no plot, a prompt to choose X and Y columns; with
  all series hidden, a note that all series are hidden.
- **FR-005b**: When a load has bad cells, the data panel MUST show a summary ("N bad
  cells") that opens the paged warning list; the summary stays available until another
  file is loaded.
- **FR-005c**: The status bar MUST show the local API address, or a message if the API
  could not start.

**Plotting**

- **FR-006**: Users MUST be able to select exactly one numeric or date/time column as X
  and one or more other numeric columns as Y.
- **FR-006a**: When X is a date/time column, the X axis MUST be a time axis whose tick
  labels are readable dates/times appropriate to the zoom level (e.g., days when zoomed
  out, seconds when zoomed in).
- **FR-007**: The system MUST draw one series per Y column, each in a distinct color, with
  a legend and axis labels taken from the column names.
- **FR-008**: Users MUST be able to switch all series between line and scatter style.
- **FR-009**: Users MUST be able to pan by Shift+dragging (or dragging with the middle
  mouse button), zoom with the scroll wheel centered on
  the pointer, and reset the view to fit all visible series.
- **FR-009a**: A plain drag on the plot MUST draw a rectangle and zoom to that region on
  release (box zoom). A drag shorter than 5 pixels is treated as a click, not a zoom.
- **FR-009b**: When the pointer is near a plotted point, the system MUST show that point's
  series name and exact X and Y values (dates/times shown in full for date/time X).
- **FR-009c**: Users MUST be able to hide and show an individual series by clicking its
  legend entry; hidden series stay selected, are marked as hidden in the legend, and are
  ignored when resetting the view.
- **FR-009d**: When a plot is first drawn, the view MUST fit all visible data with 5%
  padding on each side.
- **FR-009d2**: When Y columns are added or removed or the style is toggled (from the
  window or the API), the current X range MUST be kept and the Y range refit to the
  visible data in that X range. Changing the X column MUST reset the view (FR-009d).
  Resizing the window MUST keep both ranges.
- **FR-009e**: The view MUST be reset by a "Reset view" button, by double-clicking the
  plot, and by a keyboard shortcut.
- **FR-009f**: Each wheel notch MUST zoom by a factor of 1.2. Zooming in MUST stop at a
  visible range of about one millionth of the data's full range (per axis), and zooming out
  at 100 times the full range.
- **FR-009g**: The hover readout MUST pick the visible point closest to the pointer
  within 8 pixels across all visible series; hidden series and missing values are never
  reported. With no point within 8 pixels, no readout is shown.
- **FR-009h**: Axes MUST show about one labeled tick per 80 pixels at "nice" values.
  Numeric labels MUST switch to scientific notation when absolute values are ≥ 10^6 or
  < 10^-3 (excluding zero). Time-axis labels MUST adapt across spans from sub-second to
  multi-year, showing enough context (e.g., the date at day boundaries) to read an
  absolute time.
- **FR-009i**: Series colors MUST come from a fixed palette of at least 8 colors that stay
  distinguishable under common color-vision deficiencies and have at least 3:1 contrast
  against the plot background in both appearances. A series keeps its color while other
  series are added or removed; beyond the palette size, colors repeat with a dashed line
  style (line) or a hollow marker (scatter).
- **FR-009j**: The legend MUST remain usable with many series (scrolls when it overflows)
  and long names (truncated with the full name on hover).
- **FR-009k**: The plot MUST be operable from the keyboard: arrow keys pan, `+`/`-` zoom
  around the center, and `0` resets the view when the plot has focus. File opening,
  column selection, style, and theme controls MUST be reachable and operable by keyboard.
- **FR-010**: The plot MUST faithfully represent the data at every zoom level: features
  of the data (such as spikes and extremes) visible when zoomed in MUST NOT disappear when
  zoomed out.

**Appearance**

- **FR-010a**: The app MUST offer Light, Dark, and System appearance, defaulting to
  System. System follows the Windows app-mode setting and updates when it changes. The
  choice MUST be remembered across restarts. Plot series colors MUST remain distinct and
  legible in both light and dark appearance.

**Errors**

- **FR-011**: Every load or plot failure MUST be reported with: what went wrong, where
  (row number and column name/position when applicable), the offending value when
  applicable, and a suggested next step.
- **FR-011a**: A value that doesn't match its column's type (e.g., text in a numeric or
  date/time column) MUST NOT fail the load. The cell
  MUST be stored as missing (NaN), and the load MUST complete with a warning giving the
  total number of bad cells, the count per column, and, for each one, its row, column,
  and original value (values longer than 64 characters are shown truncated with `…`).
  The list MUST remain usable with very large counts by showing it in pages (e.g., 100
  at a time). Structural problems (wrong field count, invalid encoding, malformed quoting,
  no data, unreadable file, insufficient memory) MUST fail the load.
- **FR-012**: Every failure and warning MUST belong to a fixed, documented set of kinds,
  each with a stable identifying code and a defined suggested next step. The set is
  listed in the feature's error-code contract (`contracts/error-codes.md`).
- **FR-013**: A failure MUST NOT terminate the app or alter previously loaded data or the
  current plot.

**Local API**

- **FR-014**: While the app is running, a local script MUST be able to: load a file by
  path; retrieve the loaded file's name, row count, and column information; set the plot
  (X column, Y columns, style); and retrieve the current plot selection and style.
- **FR-014a**: Each successful load MUST produce a new dataset ID, returned to the script
  and included in state queries, so that scripts written now keep working if multiple
  datasets are supported later.
- **FR-015**: Actions performed through the local API MUST be reflected in the open window
  immediately, and actions performed in the window MUST be visible to subsequent API
  queries. API-triggered changes MUST NOT steal focus from other applications; the window
  shows a brief notice (e.g., "Loaded by script: run1.csv").
- **FR-016**: API failures and warnings MUST return the same code, row, column, and
  message that the window would show, and the window MUST also display errors and warnings
  from API-triggered loads.
- **FR-017**: The local API MUST accept connections only from the same computer.
- **FR-018**: The local API MUST be documented well enough for a user to write a working
  load-and-plot script without reading the app's source code. The documentation MUST
  include a working Python example that needs no packages beyond Python's standard
  library.

### Key Entities

- **Dataset**: The currently loaded file — its ID (new for every load), source path and
  name, row count, and ordered list of columns.
- **Column**: A named, positioned column of a dataset — its type (numeric, date/time, or
  text), whether it can be used as X and/or Y, and how many missing values it has.
- **Plot Configuration**: The current X column, the ordered set of Y columns, and the
  style (line or scatter).
- **Load/Plot Error**: A failure report — error code, message, row (optional), column
  (optional), offending value (optional), and suggested next step.
- **Bad Cell Warning**: A cell in a numeric or date/time column that could not be read as
  that type — its row, column, and original text. The cell's value is stored as missing (NaN).

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: A file of 1 million rows × 10 numeric columns loads and is ready to plot in
  under 3 seconds on the reference machine (mid-range Windows laptop).
- **SC-002**: Panning and zooming a plot of 1 million points per series stays smooth at 60
  frames per second on the reference machine.
- **SC-003**: The app starts to an interactive window in under 2 seconds.
- **SC-004**: With a 1-million-row file loaded, the app's memory use stays under 3× the
  file's size on disk.
- **SC-007**: Every action available in the window for loading and plotting can also be
  performed by a local script (pure view actions such as zooming and hiding a series are
  exempt, per the constitution), verified by a script that reproduces Stories 1 and 2
  end-to-end.
- **SC-008**: No malformed input from the test corpus causes the app to crash.

## Assumptions

- Windows (x64) is the only supported platform, per the project constitution.
- Only CSV-style text files are in scope; other formats (Excel, Parquet, etc.) are future
  features.
- Only ISO 8601 dates and date-times are recognized; other date formats are future work.
- Decimal-comma number formats are out of scope for this feature.
- The data is read-only: no editing, filtering, or transforming values in this feature.
- Exporting plots or saving sessions is out of scope for this feature.
- Logarithmic axes are out of scope for this feature.
- Cancelling an in-progress load is out of scope for this feature.
- Only one plot of one dataset is shown at a time.
- The local API is available only while the app is running; a headless mode is out of
  scope. It requires no authentication beyond being reachable only from the same computer.
- AI-agent access (MCP) is a separate future feature built on the same local API.
- No script client package (e.g., a Python library) is published in this feature.
- "Reference machine" means a mid-range Windows laptop (about 4 CPU cores, 16 GB RAM,
  integrated graphics); CI runners serve as its proxy for automated checks, per the
  constitution's performance principle.
