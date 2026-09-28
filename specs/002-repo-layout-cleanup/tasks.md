# Tasks: Repository Layout Cleanup

**Input**: Design documents from `specs/002-repo-layout-cleanup/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, quickstart.md

**Tests**: No new tests. The existing suites are the regression net (SC-004); verification tasks run them.

## Format: `[ID] [P?] [Story] Description`

## Phase 1: Setup

- [X] T001 Record the pre-move baseline in specs/002-repo-layout-cleanup/baseline.md: per-suite pass counts from `cargo test --workspace` and `npm run test`, and the perf numbers from `cargo test -p plot-twist-core --release --test perf -- --ignored --nocapture` and `scripts/check-startup.ps1` (SC-004, SC-005)

---

## Phase 2: Foundational (moves only — blocks everything else)

- [X] T002 `git mv src-tauri desktop`
- [X] T003 `git mv crates/core core` and `git mv crates/api api`, then remove the empty `crates/` folder
- [X] T004 Commit the moves alone ("Move crates to top-level component folders"), staging only the renames (spec docs go in a separate commit), with no content edits (FR-009)

**Checkpoint**: Tree matches the final layout; the build is expected to be broken until Phase 3.

---

## Phase 3: User Story 2 - Everything still builds, runs, and passes (Priority: P1)

**Goal**: Every path reference follows the move (research R1).
**Independent Test**: `cargo test --workspace` and `npm run check` pass with baseline counts; the app runs.

- [X] T005 [US2] Set `members = ["core", "api", "desktop"]` in Cargo.toml
- [X] T006 [US2] Change path deps to `path = "../api"` and `path = "../core"` in desktop/Cargo.toml
- [X] T007 [P] [US2] Change the fixtures path from `../../fixtures/csv` to `../fixtures/csv` in core/tests/import_valid.rs and core/tests/import_malformed.rs
- [X] T008 [P] [US2] Change the fixtures path from `../../fixtures/csv` to `../fixtures/csv` in api/tests/acceptance.rs
- [X] T009 [P] [US2] Replace `src-tauri/gen/` with `desktop/gen/` in .gitignore
- [X] T010 [P] [US2] Replace `src-tauri/gen/` and `src-tauri/icons/` with `desktop/gen/` and `desktop/icons/` in .prettierignore
- [X] T011 [P] [US2] Replace `src-tauri/**` and `crates/**` with `desktop/**`, `core/**`, `api/**` in the ignores of eslint.config.js
- [X] T012 [US2] Run `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, and `npm run check`; compare counts with baseline.md (SC-004)
- [X] T013 [US2] Run `npm run tauri info` and `npm run tauri dev`: open fixtures/csv/sensors.csv, plot two Y columns, then run fixtures/scripts/load_and_plot.py against the open app (FR-012, research R4)

**Checkpoint**: The restructured repo builds and behaves as before.

---

## Phase 4: User Story 1 - A newcomer understands the repository from its top level (Priority: P1)

**Goal**: The top level names its contents, documented for newcomers.
**Independent Test**: `ls -d */` shows only role-named folders; each thematic bucket is at `<component>/src/<bucket>/`.

- [X] T014 [US1] Add a "Repository layout" section to README.md naming each top-level folder (`desktop/`, `core/`, `api/`, `ui/`, `docs/`, `fixtures/`, `scripts/`, `specs/`) in one line each, noting the `<component>/src/` shape (FR-010)
- [X] T015 [US1] Check the root listing and bucket depth against quickstart.md §1, and confirm exactly four `Cargo.toml` files (workspace + `core`, `api`, `desktop`) and one `package.json` (FR-002, FR-003, FR-006, SC-001, SC-002)

---

## Phase 5: User Story 3 - No stray or duplicate tooling folders (Priority: P2)

**Goal**: One `node_modules/`, one build-output root (research R2, R3).
**Independent Test**: After install, dev, test, and build, only `./node_modules` exists and no root `dist/`.

- [X] T016 [US3] In vite.config.ts set `build.outDir` to `"../target/ui"` and add `cacheDir: "../node_modules/.vite"` (FR-004, FR-005)
- [X] T017 [US3] Set `frontendDist` to `"../target/ui"` in desktop/tauri.conf.json
- [X] T018 [P] [US3] Remove the `dist/` entries from .gitignore, .prettierignore, and eslint.config.js; add `target/**` to the eslint.config.js ignores
- [X] T019 [US3] Delete the local untracked leftovers `dist/`, `ui/node_modules/`, and any remaining `src-tauri/` (e.g. `src-tauri/gen/`)
- [X] T020 [US3] Add a note to the README "Development" section: checkouts from before the move can delete `dist/`, `ui/node_modules/`, and `src-tauri/` (FR-010)
- [X] T021 [US3] In a fresh `git worktree` of the branch (clean clone state), run `npm ci`, `npm run build`, `npm run test`, `npm run dev` (stop it), and `npm run tauri build`; confirm `target/ui/index.html` exists, no root `dist/`, and exactly one `node_modules/` (SC-003)

---

## Phase 6: User Story 4 - The constitution describes the layout (Priority: P2)

**Goal**: Principle VII governs the repository root (research R6).
**Independent Test**: The repo satisfies amended Principle VII; version 1.1.0.

- [X] T022 [US4] Amend Principle VII in .specify/memory/constitution.md: add the repository-root rule (top-level folders are role-named components `desktop/`, `core/`, `api/`, `ui/` or support areas `docs/`, `fixtures/`, `scripts/`, `specs/`; no language/template/kind folders such as `crates/`, `packages/`, `src-tauri/`, `apps/`) and name each component's source root as `<component>/src/`; bump Version to 1.1.0 and Last Amended to 2026-09-27; draft the Sync Impact Report for review, then remove it before commit (FR-011)
- [X] T023 [US4] Review the repository against the amended Principle VII and record the result in baseline.md (US4 AC1)

---

## Phase 7: Polish & Cross-Cutting

- [X] T024 Search tracked files (excluding specs/001-csv-plot-explorer/ and lockfiles) for `crates/`, `src-tauri`, and `dist` and fix any stale reference (FR-010)
- [X] T025 Run the full CI-equivalent sequence from .github/workflows/ci.yml locally (build, fmt, clippy, test, check, perf, startup); record perf and startup numbers in baseline.md and confirm they are within 10% of the baseline (SC-004, SC-005, FR-008)
- [X] T026 Run `git log --follow --oneline -- core/src/lib.rs` and `-- desktop/src/main.rs` and confirm pre-move commits appear; run `git diff -M --summary main...HEAD` and confirm every moved file shows as a rename, which is what the squash commit will record (FR-009, SC-006)

---

## Dependencies & Execution Order

- Phase 1 → Phase 2 → Phase 3 (US2 restores the build) → Phases 4, 5, 6 (independent of each other) → Phase 7.
- T004 must precede every content edit so the move commit stays pure (FR-009).
- Within Phase 3, T007–T011 are [P] (different files); T012–T013 run after them.

## Parallel Example

```text
T007 core tests fixture path | T008 api test fixture path | T009 .gitignore | T010 .prettierignore | T011 eslint.config.js
```

## Implementation Strategy

MVP = Phases 1–3: the moved layout that builds and behaves identically. Phases 4–6 add docs,
the tooling-folder cleanup, and the constitution amendment; Phase 7 is the full verification.
