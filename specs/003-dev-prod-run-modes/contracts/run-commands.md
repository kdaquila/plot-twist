# Contract: Run Commands (contributor interface)

All commands run from the repository root after `npm ci`.

| Command | Mode | Result |
|---|---|---|
| `npm run tauri dev` | Dev | Starts Vite on port 5173, builds a development build, opens the app. Frontend edits hot-reload; backend edits (`desktop/`, `api/`, `core/`) rebuild and restart the app. Ctrl+C stops both. |
| `npm run tauri build` | Prod | Builds the frontend into `target/ui/`, then `target/release/plot-twist.exe` (standalone) and `target/release/bundle/nsis/plot-twist_<version>_x64-setup.exe` (installer). |
| `npm run tauri build -- --no-bundle` | Prod | Standalone `target/release/plot-twist.exe` only. |
| `cargo run` (unsupported way to start the app) | Dev build without Vite | Window shows "The dev server is not running" and names `npm run tauri dev`. |

## Failure outputs

| Situation | Terminal | Window |
|---|---|---|
| Port 5173 taken by another program during `npm run tauri dev` | Vite: `Error: Port 5173 is already in use`; Tauri: `beforeDevCommand terminated with a non-zero status code` (exit 1) | "Port 5173 is used by another program" if the app window was opened |
| Backend edit does not compile during dev | `error[E…]` from rustc; the watcher keeps running | Previous app instance stays until the next good save |
| A plot-twist copy (dev or installed) is already running | Nothing new | The existing window comes to the front (single instance) |
