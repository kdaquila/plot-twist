# Research: Dev and Prod Run Modes

## R1. Why `cargo run` shows "can't reach this page"

**Finding**: Tauri compiles two kinds of app. Without the `custom-protocol` Cargo feature the
build is a *development build* (`tauri::is_dev()` is `true`, `cfg(dev)`): its window loads
`build.devUrl` (`http://localhost:5173`). `npm run tauri dev` starts the Vite dev server
(`beforeDevCommand`), waits for the URL to answer, then runs `cargo run --no-default-features`.
A bare `cargo run` (or `cargo run --release`) also produces a development build, but nothing
starts Vite, so WebView2 shows its "localhost refused to connect" page. `npm run tauri build`
adds `custom-protocol`; that build serves the frontend from `frontendDist` (`target/ui/`)
embedded in the executable and never contacts a dev server.

Reproduced 2026-09-27 on `main` (screenshot in the session): `cargo run` → WebView2 error page;
`npm run tauri dev` → working app.

## R2. How the development build shows a message instead (FR-003)

**Decision**: In the startup `setup` hook, when `tauri::is_dev()` is true, check whether the
configured `devUrl` answers. If not, navigate the `main` window to an inline `data:` HTML page
that says the dev server is not running and names `npm run tauri dev`. New file
`desktop/src/startup/dev_server.rs` (~90 lines); one call from `startup::setup`.

- The config windows already exist when `setup` runs, so `get_webview_window("main")` works.
- Reachability: TCP connect to every address `localhost:5173` resolves to (Vite listens on
  `::1` on Node 24, so IPv4-only would be wrong), 300 ms timeout each. A refused connection
  fails at once, so the check adds no noticeable delay in either case.
- `data:` page: percent-encoded HTML with `color-scheme: light dark` so it matches the OS
  theme. The app's CSP applies to the app's own assets, not to this page.
- Prototype verified (screenshot): the message replaces the error page; no visible flash of
  the error page on a warm start.

**Rationale**: `is_dev()` is exactly "this build loads from a dev server", so the check can
never affect prod builds (FR-005, and the startup budget, which is measured on a prod build).
Reading `devUrl` from config keeps one source of truth for the port.

**Alternatives rejected**: fall back to the last `target/ui` build (Option B, declined by the
user: can show stale screens); README only (Option C, declined); a `create: false` window built
in code (changes prod startup for a dev-only concern).

## R3. Port 5173 already in use (FR-008)

**Finding** (reproduced 2026-09-27 with a Node server on `localhost:5173`): Vite fails with
`Error: Port 5173 is already in use` (`strictPort: true`), and `tauri dev` exits 1 with
"The beforeDevCommand terminated with a non-zero status code". But the Tauri CLI had already
seen the URL answer (the other program) and launched the app, and the app window stays open
after the CLI exits, showing the other program's page.

**Decision**: Extend the R2 check from "does it answer?" to "is it plot-twist?": send
`GET / HTTP/1.1` over the same TCP connection and look for `<title>plot-twist</title>` (from
`ui/index.html`) in the reply. Three outcomes:

| Outcome | Page shown |
|---|---|
| Answers with plot-twist's page | Normal app (no change) |
| Nothing answers | "The dev server is not running" + `npm run tauri dev` |
| Something else answers | "Port 5173 is used by another program" + close it, then `npm run tauri dev` |

The window name/port come from `devUrl`. Vite's own terminal error already names the port, so
the terminal side of FR-008 needs no change.

**Rationale**: Keeps the whole fix in one small dev-only module; the CLI's launch order can't
be changed from the project.

**Alternatives rejected**: a wrapper npm script that checks the port before `tauri dev` (changes
the documented command the user asked about); `strictPort: false` (Vite would move ports while
the app still loads 5173).

## R4. Prod mode commands and output

**Decision**: document existing commands, add no npm scripts (Principle VI).

| Goal | Command | Output |
|---|---|---|
| Installer + standalone app | `npm run tauri build` | `target/release/bundle/nsis/plot-twist_0.1.0_x64-setup.exe` and `target/release/plot-twist.exe` |
| Standalone app only (quick check) | `npm run tauri build -- --no-bundle` | `target/release/plot-twist.exe` |

`beforeBuildCommand` (`npm run build`) builds the frontend into `target/ui/` first, which is
embedded; the exe runs anywhere without the checkout. Verified in feature 002 (clean-clone
`tauri build`, installer 2.81 MiB); re-verified in this feature from a clean clone (FR-009).

## R5. Transient compile errors and single instance

- A save that leaves Rust code uncompilable makes `tauri dev` print a compile error and keep
  watching; the next good save rebuilds and relaunches (seen in the contributor's terminal:
  E0599 then "Finished"). No code change; README troubleshooting note (FR-007).
- The single-instance plugin keys on the bundle identifier, which dev and prod builds share. A
  running installed copy therefore swallows a dev launch (the dev app focuses it and exits). No
  code change; README note (edge case).

## R6. Hot reload (SC-004)

Vite HMR updates React components in place on save; backend changes restart the app via the
Tauri CLI watcher over `desktop/`, `api/`, `core/`. Measured in verification.
