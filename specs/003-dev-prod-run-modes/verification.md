# Verification: Dev and Prod Run Modes

Steps: [quickstart.md](quickstart.md). Results below, recorded 2026-09-27 on Windows 11 x64.

## Baseline (`main` at b645d0b, before this feature; T001)

Measured in a separate `git worktree` of `main`.

| Measure | Result |
|---|---|
| `cargo test --workspace` | 48 passed, 1 ignored (api acceptance 6, core lib 21, import_malformed 8, import_valid 8, view_faithfulness 5; perf ignored) |
| `npm run test` | 15 passed (3 files) |
| Perf (1M rows × 10 cols) | load 0.488 s, view 13.3 ms, memory 0.84× file |
| `scripts/check-startup.ps1` | median 0.95 s (runs 1.04, 0.95, 0.95) |

## Dev-server check (T017, throwaway)

`probe` validated by 4 automated tests with fake servers on ephemeral localhost ports (plot-twist
page → `Ours`; other reply → `Other`; nothing listening → `Missing`; `localhost` resolving to
both families → `Ours`, and loopback-host guard). All passed; module deleted afterwards
(Constitution III). `cargo fmt --check` and `clippy -D warnings` clean.

## Dev mode (US1)

| Step | Result |
|---|---|
| T005 `target/debug/plot-twist.exe` with no dev server | "The dev server is not running" page naming `npm run tauri dev` within 4 s (screenshot); log `state: Missing`; `api.json` rewritten, so the local API started |
| T006 same exe, port 5173 held by a Node server | "Port 5173 is used by another program" page (screenshot); log `state: Other` |
| T006 `npm run tauri dev`, port held | Vite `Error: Port 5173 is already in use`; Tauri `beforeDevCommand terminated with a non-zero status code`; exit 1; the window the CLI had launched shows the port page (log `state: Other`) |
| T007 `npm run tauri dev` | App window; log `state: Ours`, `ready`. `load_and_plot.py fixtures/csv/sensors.csv time temp pressure` → 240 rows, temp and pressure plotted in the window |
| T007 hot reload | Toolbar label edited in `ui/src/app/Toolbar.tsx`: visible 1.5 s after save, loaded plot kept (no restart) (screenshot); edit reverted |
| T007 backend change | Edit in `core/src/lib.rs` → "File core\src\lib.rs changed. Rebuilding application..." → app relaunched; edit reverted |

## Prod mode (US2)

| Step | Result |
|---|---|
| T008 `npm run tauri build` | `target/release/plot-twist.exe` (12.2 MB) and `target/release/bundle/nsis/plot-twist_0.1.0_x64-setup.exe` (2.81 MiB) |
| T008 run the exe, nothing on port 5173 | App window; `load_and_plot.py … time temp pressure` loaded 240 rows and plotted (screenshot); 0 dev-server log lines (check skipped in prod builds) |
| T009 installer (`/S`, per-user) | Installed to `%LOCALAPPDATA%\plot-twist\` with a Start menu shortcut; started from the shortcut, ran from the installed path; script loaded and plotted humidity; `uninstall.exe /S` removed the exe and the shortcut (logs and `api.json`, which are user data, remain) |
| T010 `npm run tauri build -- --no-bundle` | `target/release/plot-twist.exe` only; no `bundle/` folder |

## Clean clone (T014)

`git worktree add --detach ../pt-verify HEAD` at 8fbbb58: no `node_modules/`, no `target/`.
Followed only the README.

| Step | Result |
|---|---|
| `npm ci` | 235 packages |
| `npm run tauri dev` | first build 1m 55s; app window, log `state: Ours`, `ready`; `load_and_plot.py` plotted temp and pressure |
| `npm run tauri build` | exe and `plot-twist_0.1.0_x64-setup.exe` (2.81 MiB) |
| Run `target/release/plot-twist.exe`, nothing on port 5173 | script plotted temp and pressure |

Worktree removed afterwards.

## CI sequence and budgets (T015)

| Check | Result | vs baseline |
|---|---|---|
| `npm run build` | ok | — |
| `cargo fmt --all --check` | ok | — |
| `cargo clippy --workspace --all-targets -D warnings` | ok | — |
| `cargo test --workspace` | 48 passed, 1 ignored | same |
| `npm run check` | ok; 15 tests passed | same |
| Perf | load 0.503 s, view 14.4 ms, memory 0.84× | load +3.1% (within 10%) |
| `scripts/check-startup.ps1` | median 0.97 s | +2.1% (within 10%) |
