# Implementation Plan: Repository Layout Cleanup

**Branch**: `002-repo-layout-cleanup` | **Date**: 2026-09-27 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `specs/002-repo-layout-cleanup/spec.md`

## Summary

Move the three Rust crates to top-level folders named for their role — `desktop/` (was
`src-tauri/`), `core/` and `api/` (were under `crates/`) — so every component, including the
unchanged `ui/`, has the shape `<component>/src/<thematic buckets>/`. Send the frontend build
output to `target/ui/` and the dev-server cache to the root `node_modules/`, leaving one
dependency folder and one build-output folder. Update every path reference (research R1),
document the layout in the README, and amend constitution Principle VII to govern the
repository root (1.0.2 → 1.1.0). No behavior changes.

## Technical Context

**Language/Version**: Rust 1.98 (edition 2024, workspace resolver 3); TypeScript 6 strict; Node 24

**Primary Dependencies**: Tauri 2.12 (CLI + `tauri-build`), Vite 8, Vitest, ESLint 10, Prettier, ts-rs

**Storage**: N/A

**Testing**: `cargo test --workspace`, `npm run check` (typecheck, lint, format, vitest, bindings drift), perf tests (`--ignored`), `scripts/check-startup.ps1`

**Target Platform**: Windows x64

**Project Type**: desktop app (Tauri shell + Rust engine + local HTTP API + web frontend)

**Performance Goals**: unchanged from feature 001; SC-005 allows ±10% (startup median 1.09 s, 1M-row load 0.51 s)

**Constraints**: history preserved through the move (R5); all command names unchanged (FR-007)

**Scale/Scope**: ~100 tracked files move; ~12 config/path edits; 1 constitution amendment; README section

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Check | Status |
|---|---|---|
| I. Backend owns the data; every client equal | No data operation added or changed; `core` stays free of GUI and web-server deps (separate crates kept). | Pass |
| II. Type safety as a quality gate | `tsc` strict, clippy `-D warnings`, and the ts-rs bindings drift check all keep running after the move. | Pass |
| III. Validate with tests | No behavior change; existing suites must pass with 0 tests lost against a recorded baseline (SC-004). No throwaway tests. | Pass |
| IV. Performance budgets | Re-measured after the move (SC-005). | Pass |
| V. Errors: closed vocabulary, no panics | No error-path code touched. | Pass |
| VI. Simplicity | Removes a folder level and two stray folders; adds no abstraction. | Pass |
| VII. Code Organization | The amendment extends VII to the repository root; the new layout satisfies the amended rules (depth 3 to every thematic bucket, within the 3–4 cap below each source root unchanged). | Pass |
| Project constraints / quality gates | CI must pass unchanged (FR-008). | Pass |

Post-design re-check: unchanged — Pass.

## Project Structure

### Documentation (this feature)

```text
specs/002-repo-layout-cleanup/
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── checklists/requirements.md
└── tasks.md            (/speckit-tasks)
```

No `contracts/`: the feature changes no external interface (GUI, local API, and Tauri
commands are untouched).

### Source Code (repository root) — after this feature

```text
desktop/                 Tauri shell            (git mv src-tauri → desktop)
├── src/{commands,events,startup}/, lib.rs, main.rs
├── capabilities/  icons/  build.rs  Cargo.toml  tauri.conf.json
core/                    data engine            (git mv crates/core → core)
├── src/{csv_import,dataset,errors,session,settings,view}/, lib.rs
├── examples/  tests/  Cargo.toml
api/                     local HTTP API         (git mv crates/api → api)
├── src/{discovery,routes}/, lib.rs
├── tests/  Cargo.toml
ui/                      frontend               (unchanged)
├── src/{app,backend,data-panel,plot}/, main.tsx
└── index.html
docs/  fixtures/  scripts/  specs/
Cargo.toml  package.json  vite.config.ts  tsconfig.json  eslint.config.js  … (root configs)

gitignored: node_modules/ (only one), target/ (only build-output root; frontend in target/ui/)
```

**Structure Decision**: The tree above (spec Clarifications, "Final layout").

## Implementation Approach

1. **Move commit** — `git mv src-tauri desktop`, `git mv crates/core core`,
   `git mv crates/api api`, remove the empty `crates/`. No content edits in this commit.
2. **Reference commit** — apply research R1 edits (workspace members, path deps, fixture paths
   in tests, Vite `outDir`/`cacheDir`, Tauri `frontendDist`, ignore lists, ESLint ignores).
3. **Local cleanup** — delete the untracked old `dist/`, `ui/node_modules/`, and
   `src-tauri/gen/` leftovers; rebuild; confirm exactly one `node_modules/` and no `dist/`.
4. **Docs + constitution** — README "Repository layout" section and old-cache note;
   Principle VII amendment 1.0.2 → 1.1.0 (R6).
5. **Verify** — `cargo fmt/clippy/test`, `npm run check`, perf tests, startup check,
   `npm run tauri build`, app smoke run (load + plot via GUI and API), `git log --follow`.

## Complexity Tracking

No violations.
