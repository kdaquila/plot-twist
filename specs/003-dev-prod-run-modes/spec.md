# Feature Specification: Dev and Prod Run Modes

**Feature Branch**: `003-dev-prod-run-modes`
**Created**: 2026-09-27
**Status**: Draft
**Input**: User description: "Let's make sure we have both the dev mode and prod mode of the app working. Let's update the README.md to explain how to run both. Also, let's address the error displayed when running 'npm run tauri dev' (I might have that command incorrect slightly)"

> The "users" of this feature are contributors (human or agent) running the app from a
> checkout. Commands appear in this spec because they are the interface being specified.

## Current State (audit, 2026-09-27, on `main` at b645d0b)

| Way of starting the app | What happens today | Verdict |
|---|---|---|
| `npm run tauri dev` | Starts the frontend dev server, compiles the backend, opens the window with the working app. | Works. |
| `npm run tauri dev` **while backend code is mid-edit** | The contributor's terminal history shows a compile error (`no method named view_with_columns`) printed once, then an automatic rebuild that succeeded. The error came from a half-saved edit during feature 001 and does not reproduce now. | Not a defect today, but nothing tells the contributor that the first error was transient. |
| `cargo run` (the contributor's terminal history shows this too) | Builds and opens the window, but the window shows the browser error page **"Hmmm… can't reach this page — localhost refused to connect (ERR_CONNECTION_REFUSED)"**, because a development build loads its screens from the frontend dev server, which only `npm run tauri dev` starts. | **Reproduced. This is the error the contributor sees.** |
| `npm run tauri build`, then run the result | Produces an installer and a standalone app that runs without any dev server (verified in feature 002). | Works; undocumented. |
| README | Documents only `npm ci` and `npm run tauri dev`. Nothing about production builds, and nothing about `cargo run`. | Gap. |

## Clarifications

### Session 2026-09-27

- Q: What should `cargo run` (a dev build started without the dev server) show instead of the browser error page? → A: A clear in-window message saying the dev server is not running and to start the app with `npm run tauri dev` (dev builds only; no fallback to an old frontend build).

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Run the app in dev mode without errors (Priority: P1)

A contributor starts the app to work on it. They run one documented command and get the
working app in a window, and their saved changes to the screens appear without restarting.
If they start it the wrong way, they are told what to do instead of seeing a browser error
page.

**Why this priority**: This is the reported problem. Every contributor hits it on day one.

**Independent Test**: From a fresh clone, follow the README's dev-mode steps; the app opens
and a file can be loaded and plotted. Then start the app the way the contributor did
(`cargo run`) and observe the outcome defined in FR-003.

**Acceptance Scenarios**:

1. **Given** a fresh clone with prerequisites installed, **When** the contributor runs the
   README's dev-mode command, **Then** the app window opens showing the app (not an error
   page) and a sample CSV can be loaded and plotted.
2. **Given** the app is running in dev mode, **When** the contributor saves a change to a
   frontend file, **Then** the window shows the change without a restart.
3. **Given** the app is running in dev mode, **When** the contributor saves a change to
   backend code, **Then** the app rebuilds and restarts by itself; if a save leaves the code
   briefly uncompilable, the next successful save recovers without restarting the command.
4. **Given** a contributor starts the development build without the frontend dev server
   (for example with `cargo run`), **When** the window opens, **Then** it shows a message
   that the dev server is not running and names `npm run tauri dev` — never the bare
   "can't reach this page" browser error.

---

### User Story 2 - Build and run the app in prod mode (Priority: P1)

A contributor wants to see the app exactly as a user would get it: optimized, self-contained,
with no dev server. They build it with one documented command and run either the installer
or the built app directly.

**Why this priority**: The user asked for both modes to work; prod mode is how performance
and startup targets are measured and how releases are made.

**Independent Test**: With no dev server running, build prod mode per the README, run the
built app, load and plot a sample CSV, and run the example script against it.

**Acceptance Scenarios**:

1. **Given** a fresh clone, **When** the contributor runs the README's prod-build command,
   **Then** it produces both an installer and a standalone app, and the README names where
   each is written.
2. **Given** a completed prod build and no dev server running, **When** the contributor
   runs the built app, **Then** the app opens showing the app (not an error page), loads and
   plots a sample CSV, and serves the local API to the example script.
3. **Given** a completed prod build, **When** the contributor runs the installer and starts
   the installed app, **Then** it behaves as in scenario 2, and uninstalling it removes it.
4. **Given** the contributor only wants a quick prod check, **When** they follow the README,
   **Then** they can build the standalone app without also building the installer.

---

### User Story 3 - The README explains both modes (Priority: P2)

A contributor reading the README learns, in one place, which command runs which mode, what
each mode is for, where the prod build output lands, and what to do if they see the
"can't reach this page" error or a compile error while editing.

**Why this priority**: Documentation is how US1 and US2 stay working for the next
contributor; it depends on US1 and US2 being settled first.

**Independent Test**: A reader who has not seen this spec can, from the README alone, run
dev mode, build and run prod mode, and explain the error page if they hit it.

**Acceptance Scenarios**:

1. **Given** the README, **When** a contributor looks for how to run the app, **Then** they
   find a dev-mode section and a prod-mode section, each with its command(s) and a one-line
   statement of when to use it.
2. **Given** the README, **When** a contributor looks for the prod build output, **Then** it
   names the installer location and the standalone app location.
3. **Given** a contributor saw the "can't reach this page" error or a compile error during
   dev mode, **When** they read the README's troubleshooting note, **Then** it tells them the
   cause and the fix.

---

### Edge Cases

- The frontend dev server's port (5173) is already in use by another program: dev mode
  fails with a terminal message naming the port, and any app window that opens says the port
  is used by another program instead of showing that program's page (FR-008).
- A copy of the app is already running (the app is single-instance): starting another in
  either mode brings the existing window forward; the README says so, since a contributor
  may otherwise think their new build did not start.
- Prod build run while a dev-mode app is still open: the build still succeeds (the two
  modes do not share output that one would lock for the other).
- A backend edit that leaves the code uncompilable during dev mode: the command keeps
  running and recovers on the next good save (US1 scenario 3).

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: Dev mode MUST start with a single command from the repository root after
  installing dependencies, and MUST open the working app, not an error page.
- **FR-002**: In dev mode, saved frontend changes MUST appear without restarting, and saved
  backend changes MUST trigger an automatic rebuild and restart.
- **FR-003**: Starting the development build without the frontend dev server MUST NOT show
  the bare browser error page. Instead the window MUST show a plain in-window message saying
  the dev server is not running and naming the dev-mode command (`npm run tauri dev`). The
  rest of the app, including the local API, starts as usual. The message stays until the app
  is restarted; it need not recover by itself when the dev server starts later. The check
  contacts only the configured dev-server address on localhost. This applies to development
  builds only; prod builds never check for or depend on a dev server.
- **FR-004**: Prod mode MUST be buildable with a single command, producing a standalone app
  and an installer; a documented variant MUST build only the standalone app.
- **FR-005**: The prod-mode app MUST run with no dev server running and without the
  repository present (the installed copy), and MUST pass the same load-plot-script check as
  dev mode.
- **FR-006**: The README MUST have a "dev mode" and a "prod mode" section, each with the
  command(s), when to use the mode, and (for prod) where the installer and standalone app are
  written. The existing CI-check list and Spec Kit setup notes MUST stay.
- **FR-007**: The README MUST include a troubleshooting note covering, each with cause and
  fix: the dev-server message (or the "can't reach this page" error on older checkouts), the
  dev-server port being in use, a transient compile error while editing, and an
  already-running copy (dev or installed) taking over a new launch.
- **FR-008**: Dev mode MUST fail with a message naming the port when the dev-server port is
  already taken, and any app window opened in that case MUST say the port is used by another
  program instead of showing that program's page (edge case).
- **FR-009**: Both modes MUST be verified end to end from a clean clone before this feature
  is complete. The steps MUST be recorded in the feature folder so they can be repeated, and
  the results recorded next to them.
- **FR-010**: Existing checks (build, format, lint, tests, performance and startup budgets)
  MUST still pass.

### Key Entities

- **Dev mode**: the app running from a development build, with its screens served live by
  the frontend dev server and automatic rebuilds on save.
- **Development build**: any build not made by the prod-build command, including a plain
  `cargo run` or `cargo run --release`; it always expects the dev server.
- **Prod mode**: the app built optimized and self-contained, with its screens packaged
  inside it; delivered as a standalone app and as an installer.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: From a fresh clone with prerequisites installed, a contributor following only
  the README reaches a working dev-mode window with a plotted sample CSV in one attempt,
  with 0 error pages or unexplained errors.
- **SC-002**: From a fresh clone, a contributor following only the README builds prod mode
  and runs the result with no dev server, loading, plotting, and scripting a sample CSV in
  one attempt.
- **SC-003**: Starting the development build without the dev server shows 0 browser error
  pages; the window names the dev-mode command within 5 seconds of opening.
- **SC-004**: A frontend change saved in dev mode is visible in the window within 2 seconds
  without a restart.
- **SC-005**: All existing checks pass with the same test counts as before this feature, and
  performance and startup stay within 10% of a baseline measured on `main` before this
  feature's changes.

## Assumptions

- "Dev mode" is `npm run tauri dev` (the contributor's command was correct) and "prod mode"
  is the optimized, self-contained build a user would install.
- The compile error in the contributor's terminal history was a one-off from a half-saved
  edit during feature 001; it does not reproduce on current `main`, so no code fix is needed
  for it beyond the README note (FR-007).
- Code signing of the installer, auto-update, and publishing releases are out of scope.
- Windows (x64) only, as for the rest of the project.
- A "clean clone" is a fresh checkout of the branch with no `node_modules/`, no `target/`,
  and no other untracked files; prerequisites (Rust, Node, WebView2) are installed.
- The README remains the single place contributors look for how to run the app; no separate
  contributing guide is added.
