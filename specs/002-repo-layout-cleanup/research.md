# Research: Repository Layout Cleanup

All decisions below were checked against the repository as of `15699e6` (main after feature 001).

## R1. Path references that must follow the move

A `git grep` for `crates/`, `src-tauri`, `dist`, `../`, `ui/src` over tracked files (excluding
`specs/001-*` and lockfiles) found every reference:

| File | Reference | New value |
|---|---|---|
| `Cargo.toml` | `members = ["crates/core", "crates/api", "src-tauri"]` | `["core", "api", "desktop"]` |
| `api/Cargo.toml` | `path = "../core"` | unchanged (siblings stay siblings) |
| `desktop/Cargo.toml` | `path = "../crates/api"`, `"../crates/core"` | `"../api"`, `"../core"` |
| `core/tests/import_valid.rs`, `import_malformed.rs`, `api/tests/acceptance.rs` | `CARGO_MANIFEST_DIR/../../fixtures/csv` | `../fixtures/csv` |
| `desktop/tauri.conf.json` | `frontendDist: "../dist"` | `"../target/ui"` |
| `vite.config.ts` | `outDir: "../dist"` (relative to `root: "ui"`) | `"../target/ui"`; add `cacheDir: "../node_modules/.vite"` |
| `.gitignore` | `dist/`, `src-tauri/gen/` | drop `dist/` (under `target/`); `desktop/gen/` |
| `.prettierignore` | `dist/`, `src-tauri/gen/`, `src-tauri/icons/` | drop `dist/`; `desktop/gen/`, `desktop/icons/` |
| `eslint.config.js` ignores | `dist/**`, `src-tauri/**`, `crates/**` | `target/**`, `desktop/**`, `core/**`, `api/**` (flat config ignores only `node_modules/` and `.git/` by default) |
| `.cargo/config.toml` | `TS_RS_EXPORT_DIR = "ui/src/backend/generated"` | unchanged (`ui/src` stays) |
| `package.json`, `tsconfig.json` | `ui/src…` | unchanged |
| `desktop/capabilities/default.json` | `../gen/schemas/…` | unchanged (relative within the crate) |
| `.github/workflows/ci.yml`, `scripts/check-startup.ps1` | no folder paths (`target\release\plot-twist.exe` is unaffected) | unchanged |

**Decision**: Apply exactly these edits. **Rationale**: exhaustive grep; nothing else names a
moved folder. **Alternatives**: none.

## R2. Where the frontend build output goes

**Decision**: `target/ui/` (Vite `outDir: "../target/ui"`, Tauri `frontendDist: "../target/ui"`).
**Rationale**: one gitignored build-output root (FR-005). `cargo clean` removing it is harmless
(`npm run build` recreates it; CI already builds the frontend before cargo because
`tauri-build` checks `frontendDist` exists). **Alternatives**: `ui/dist/` — still a second
output folder, only moved; rejected.

## R3. Stray `ui/node_modules/`

**Decision**: set Vite `cacheDir: "../node_modules/.vite"` (resolved relative to Vite's `root`,
which is `ui/`). Vitest derives its cache from Vite's `cacheDir`, so both land in the root
`node_modules/`. **Rationale**: the folder only ever held `.vite/`. **Alternatives**: move
`vite.config.ts` into `ui/` — then `root` moves too and more config shifts; rejected.

## R4. Tauri CLI finds a renamed shell folder

**Decision**: rename `src-tauri/` → `desktop/` with no extra config. **Rationale**: Tauri 2's CLI
locates the app by searching the working tree (gitignore-aware) for `tauri.conf.json`, not by
the folder name. Verified during implement with `npm run tauri info` and `npm run tauri build`.
**Alternatives**: none needed.

## R5. Preserving history

**Decision**: do the moves with `git mv` in their own commit before any content edits; the PR
is squash-merged, and the squashed commit still records renames because moved files are
≥ 90% similar (most 100%) and the change is far below git's rename-detection limit.
**Rationale**: SC-006. Verify with `git log --follow --oneline -- core/src/lib.rs` after merge
(pre-merge on the branch). **Alternatives**: none.

## R6. Constitution amendment

**Decision**: Principle VII gains a repository-root rule and names the source roots:
top-level folders are components named for their role (`desktop/`, `core/`, `api/`, `ui/`) or
support areas (`docs/`, `fixtures/`, `scripts/`, `specs/`); no folder named for a language,
framework template, or generic kind (`crates/`, `packages/`, `src-tauri/`, `apps/`); every
component's source root is `<component>/src/`, where rules 1–4 apply. Version 1.0.2 → 1.1.0
(MINOR: materially expanded guidance). Sync Impact Report added for review and removed
before commit, per Governance. **Alternatives**: PATCH — rejected; it adds a rule.

## R7. Existing checkouts

**Decision**: README "Development" notes that checkouts from before the move can delete the
old `dist/` and `ui/node_modules/` folders. Also add a short "Repository layout" section to the
README listing each top-level folder in one line (US1). **Alternatives**: a cleanup script —
unnecessary for two folders.
