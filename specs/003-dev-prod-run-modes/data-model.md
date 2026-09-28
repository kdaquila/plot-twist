# Data Model: Dev and Prod Run Modes

No runtime data or persisted state changes.

## Build kind

| Kind | How it is produced | Where the screens come from | `tauri::is_dev()` |
|---|---|---|---|
| Development build | `npm run tauri dev`, or any plain `cargo run` / `cargo build` | Vite dev server at `devUrl` (`http://localhost:5173`) | `true` |
| Prod build | `npm run tauri build` (adds the `custom-protocol` feature) | `target/ui/`, embedded in the executable | `false` |

## Dev-server state (development builds only)

Determined once at startup by `desktop/src/startup/dev_server.rs`.

| State | Detected when | Window shows |
|---|---|---|
| `Ours` | A server at `devUrl` answers `GET /` with a page containing `<title>plot-twist</title>` | The app (unchanged) |
| `Missing` | No address for `devUrl` accepts a connection within 300 ms | "The dev server is not running" + `npm run tauri dev` |
| `Other` | A server answers but the page is not plot-twist's | "Port 5173 is used by another program" + close it, then `npm run tauri dev` |

No transitions: the state is checked once; the contributor restarts the app after fixing the cause.
