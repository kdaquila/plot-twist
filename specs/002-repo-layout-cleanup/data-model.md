# Data Model: Repository Layout Cleanup

No runtime data changes. The "entities" are the repository's structural units.

## Component

| Field | Value rules |
|---|---|
| Folder | Top-level, named for its role: `desktop`, `core`, `api`, `ui`. |
| Manifest | Exactly one: `Cargo.toml` (Rust) or the root `package.json` (frontend, shared with root tooling). |
| Source root | `<folder>/src/`. Its direct children are the thematic buckets (Principle VII rule 1). |
| Package name | Unchanged: `plot-twist` (desktop), `plot-twist-core`, `plot-twist-api`. |

Dependencies (unchanged): `desktop → api → core`, `desktop → core`; `ui` talks to `desktop` over
IPC and never links against a crate. `core` depends on neither `desktop` nor `api`.

## Support area

`docs/`, `fixtures/`, `scripts/`, `specs/` — top-level, named for contents, not components.

## Generated location

| What | Where | Tracked? |
|---|---|---|
| Rust + frontend build output | `target/` (frontend in `target/ui/`) | No |
| Frontend dependencies + dev-server/test cache | `node_modules/` (cache in `node_modules/.vite/`) | No |
| Tauri schemas | `desktop/gen/` | No |
| ts-rs bindings | `ui/src/backend/generated/` | Yes (drift-checked) |
