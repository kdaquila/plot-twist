# Feature Specification: Repository Layout Cleanup

**Feature Branch**: `002-repo-layout-cleanup`
**Created**: 2026-09-27
**Status**: Draft
**Input**: User description: "Cleanup: multiple package.json files (maybe), more than one node_modules folder, multiple Cargo.toml files. Restructure the top-level folders so they readily show what they contain — less nesting, no generic folder followed by a `src` before the semantic names appear. Amend the constitution if needed. Think about it and propose options."

> This feature's subject *is* the repository's folder layout, so folder names appear in this
> spec as the thing being specified, not as implementation detail. The "users" are
> contributors (human or agent) reading and building the repository.

## Current State (audit, 2026-09-27)

| Concern raised | What is actually there | Verdict |
|---|---|---|
| Multiple `package.json` | One tracked manifest (root) and one lockfile. | Not a problem. |
| Multiple `node_modules` | Root `node_modules/` (real dependencies) and `ui/node_modules/`, which holds only the frontend dev server's cache (`.vite/`). It appears because the dev server treats `ui/` as its root. | Real, cosmetic — a stray tool cache. |
| Multiple `Cargo.toml` | Four: one workspace manifest plus one per component (`crates/core`, `crates/api`, `src-tauri`). Each separately built component needs its own. | Inherent to having three components; only merging components reduces the count. |
| Generic top-level folders | `crates/` (a language-kind name, not a thematic one) and `src-tauri/` (a framework-template name). The desktop app is the only component *not* under `crates/`. | Real. |
| `src` before semantic names | Data-import code sits at `crates/core/src/csv_import/` — the first meaningful name is 4 folders deep. Frontend: `ui/src/plot/` — 3 deep. | Real. |
| Other root clutter | `dist/` (frontend build output) sits at the root beside `target/` (backend build output): two generated folders instead of one. | Real, minor. |

## Clarifications

### Session 2026-09-27

- Q: Which target layout? → A: Option A, amended after previewing the tree (below).
- Q: Amendments to Option A? → A: Name the desktop shell `desktop/` (not `app/`, which also collides with the frontend's `app/` bucket). Keep `ui/src/` so every component uses the same `<component>/src/<thematic>/` shape as the Rust crates, for consistency. The frontend folder is therefore unchanged; only the Rust side and the stray tooling folders move.

**Final layout**: top level `desktop/` (was `src-tauri/`), `core/` (was `crates/core/`), `api/` (was `crates/api/`), `ui/` (unchanged); every component is `<component>/src/<thematic buckets>/`.

## Layout Options

> Proposal record. The options below are as first proposed; the chosen layout is Option A
> **as amended** in Clarifications (`desktop/` not `app/`; `ui/src/` kept). Where they differ,
> Clarifications wins.

Depth figures count folders from the repository root to the first thematic folder
(e.g. the data-import code).

**Option A — Name by role, keep each ecosystem's `src` convention for Rust (recommended)**

```text
app/     desktop shell (was src-tauri/)    app/src/startup/ …
core/    data engine   (was crates/core/)  core/src/csv_import/ …
api/     local API     (was crates/api/)   api/src/routes/ …
ui/      frontend      (was ui/src/*)      ui/plot/, ui/data-panel/ …
docs/ fixtures/ scripts/ specs/
```

Rust thematic folders at depth 3 (from 4); frontend at depth 2 (from 3). Every top-level
folder names what it is. Rust keeps the layout every Rust contributor and tool expects.

**Option B — Option A, plus no `src` anywhere**

```text
core/csv_import/, core/dataset/, core/tests/, core/Cargo.toml …
app/startup/, app/icons/, app/tauri.conf.json …
ui/plot/ …
```

Thematic folders at depth 2 everywhere. Cost: non-standard Rust layout that each component
must declare explicitly; source folders sit next to tests, examples, icons, and config, so
a component's folder mixes code and non-code.

**Option C — Two halves, one backend component**

```text
backend/   one Rust component: backend/src/csv_import/, …/api/, …/desktop/ …
frontend/  frontend/plot/ …
```

One `Cargo.toml` instead of four. Cost: gives up the enforced boundary that keeps the data
engine free of GUI and web-server dependencies — the boundary that makes GUI/API parity
(Constitution Principle I) checkable at build time — and every engine test builds the
whole desktop app.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - A newcomer understands the repository from its top level (Priority: P1)

A contributor opens the repository for the first time. Every top-level folder name tells
them what that folder holds (the desktop app, the data engine, the local API, the
frontend, docs, fixtures, scripts, specs) without having to open it, and the thematic
folders inside each part sit at one uniform depth: `<component>/src/<bucket>/`.

**Why this priority**: This is the core ask and the reason for the constitution amendment.

**Independent Test**: List the repository root; for each folder, a reader can state its
contents from the name alone. Count folders from the root to the data-import code and to
the plot code; both are at depth 3 (data import was 4).

**Acceptance Scenarios**:

1. **Given** the restructured repository, **When** a contributor lists the root, **Then** no
   top-level folder is named for a language, framework template, or generic kind
   (`crates/`, `src-tauri/`, `src/`, `packages/`).
2. **Given** the restructured repository, **When** a contributor navigates to the
   data-import code or the plot code, **Then** each is exactly 3 folders deep
   (`core/src/csv_import/`, `ui/src/plot/`).

---

### User Story 2 - Everything still builds, runs, and passes after the move (Priority: P1)

A contributor runs the same commands as before — install, develop, check, test, build the
installer — and everything works; the app behaves exactly as it did.

**Why this priority**: A restructure that breaks the build is a regression, not a cleanup.

**Independent Test**: Run the full local check and the CI pipeline on the restructured
branch; launch the app, load a CSV, plot it, and drive it through the local API.

**Acceptance Scenarios**:

1. **Given** a fresh clone of the restructured branch, **When** the documented setup and
   check commands run, **Then** all pass with the same test count as before the move.
2. **Given** the built app, **When** a user loads and plots a CSV via the GUI and via the
   local API, **Then** behavior matches feature 001.
3. **Given** the move, **When** a contributor views a moved file's history, **Then** its
   history before the move is still reachable.

---

### User Story 3 - No stray or duplicate tooling folders (Priority: P2)

After install, develop, test, and build, the repository contains exactly one dependency
folder for the frontend tooling and one location for build output.

**Why this priority**: Directly addresses the duplicate `node_modules` observation; small but
visible.

**Independent Test**: Run install, dev server, tests, and a full build from a clean clone,
then search the tree for dependency and build-output folders.

**Acceptance Scenarios**:

1. **Given** a clean clone after install, dev, test, and build, **When** the tree is
   searched, **Then** exactly one `node_modules/` exists (at the root).
2. **Given** the same, **When** the root is listed, **Then** there is a single generated
   build-output folder rather than two.

---

### User Story 4 - The constitution describes the layout we actually have (Priority: P2)

The constitution's Code Organization principle states the repository-root rule (thematic
top-level folders, no generic kind folders) alongside the existing source-root rules, so
future features keep the layout.

**Why this priority**: Without the amendment, the next feature can reintroduce the nesting.

**Independent Test**: Read Principle VII; it covers the repository root, and the
restructured repository satisfies every rule in it.

**Acceptance Scenarios**:

1. **Given** the amended constitution, **When** a reviewer checks the repository against
   Principle VII, **Then** there are no violations.
2. **Given** the amendment, **When** its version line is read, **Then** it shows 1.1.0 with an
   updated Last Amended date, and no Sync Impact Report remains in the committed file
   (Governance: the report is for review and is removed before commit).

### Edge Cases

- Generated TypeScript bindings are written by a backend test into a frontend folder; the
  generation target and the "bindings are up to date" check must follow the move.
- Build-tool generated folders (e.g. desktop-shell schemas) that are gitignored today must
  stay ignored at their new location.
- Editor, formatter, and linter ignore lists name old paths; stale entries must not silently
  start linting or formatting generated or vendor files.
- Historical specs (feature 001) name old paths; they are records and are not rewritten.
- Local caches from the old layout (old `ui/node_modules/`, old `dist/`) may linger in
  existing checkouts; the README says how to clear them.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The repository MUST be reorganized to the final layout in Clarifications:
  `desktop/`, `core/`, `api/`, `ui/` at the top level, each shaped `<component>/src/`, with no
  `crates/` or `src-tauri/` folder.
- **FR-002**: Every top-level folder MUST be named for what it contains; no top-level
  folder may be named for a language, framework template, or generic kind.
- **FR-003**: Every component's thematic source folders MUST sit at the same depth,
  `<component>/src/<bucket>/` (depth 3); none deeper than today.
- **FR-004**: The repository MUST have exactly one frontend package manifest, one lockfile,
  and one frontend dependency folder (`node_modules/` at the root); tool caches MUST NOT
  create a second dependency folder anywhere in the tree outside `target/`.
- **FR-005**: All generated build output MUST live under a single gitignored root folder,
  `target/` (frontend output in `target/ui/`).
- **FR-006**: Every component manifest count MUST be justified: one per separately built
  component plus at most one workspace manifest.
- **FR-007**: All existing commands (install, dev, check, test, bindings check, installer
  build, startup check) MUST work unchanged in name and outcome.
- **FR-008**: The CI pipeline MUST pass on the restructured branch.
- **FR-009**: File moves MUST preserve version-control history: on the branch they are made
  as moves in a commit with no content edits, and after the squash merge each moved file's
  pre-move history MUST still be followable (moved files stay similar enough for rename
  detection).
- **FR-010**: README, docs, contributor scripts, and tooling configs MUST reference only the
  new paths; historical feature specs are left as-is. The README MUST gain a "Repository
  layout" section naming each top-level folder in one line, and a note on removing
  pre-move leftovers (`dist/`, `ui/node_modules/`, `src-tauri/`) from existing checkouts.
- **FR-011**: Constitution Principle VII MUST be amended to govern the repository root
  (thematic top-level folders, no generic kind folders) and to name where the source roots
  are (`<component>/src/`), with a MINOR version bump (1.0.2 → 1.1.0) and a Sync Impact
  Report reviewed during the amendment and removed before commit, per Governance.
- **FR-012**: The app's behavior MUST be unchanged — no user-visible change in the GUI or
  the local API.

### Key Entities

- **Component**: a separately built unit (desktop app, data engine, local API, frontend),
  each with one manifest and one top-level folder.
- **Source root**: the folder inside a component whose direct children are the thematic
  buckets that Principle VII governs.

## Success Criteria *(mandatory)*

- **SC-001**: 100% of top-level folders pass a "name says what's inside" review (no
  language, template, or generic-kind names).
- **SC-002**: 100% of components place their first thematic folder at the same depth (3);
  none is deeper than before the move (data import improves from 4 to 3).
- **SC-003**: After install, dev, test, and build from a clean clone, exactly 1 dependency
  folder and 1 build-output folder exist.
- **SC-004**: The full local check and CI pass with the same number of tests as before the
  move (0 lost), compared against per-suite counts recorded before the first move commit.
- **SC-005**: Startup and load performance stay within 10% of feature 001's measurements
  (startup median 1.09 s; 1M-row load 0.51 s).
- **SC-006**: Every moved file's pre-move history is reachable with a standard
  history-following command.

## Assumptions

- Merging or splitting components is out of scope unless Option C is chosen.
- Moving the frontend's build output under the backend's existing build-output folder
  satisfies FR-005.
- The internal thematic folders (`csv_import`, `dataset`, `plot`, `data-panel`, …) keep their
  names; only their parents change.
- Root-level tool config files (formatter, linter, toolchain pins) stay at the root; tools
  look for them there and they are not folders.
- The work lands as one PR; no release or installer rename is involved.
- Only folders move; component (package) names, the app's product name, installer name, and
  all user-facing paths (local API discovery file, settings, logs) stay exactly as they are.
- Recovery: all work happens on the feature branch and merges only with CI green; if the
  branch cannot be made to build, it is abandoned and `main` is untouched.
- CI's build cache is keyed on manifest paths, so the first CI run after the move is a cold
  build; later runs are warm again. This is expected, not a regression.
- `scripts/` (contributor scripts), `fixtures/`, `docs/`, and `specs/` already name their
  contents and stay; tool-owned dot-folders (`.github/`, `.cargo/`, `.specify/`, `.claude/`)
  are out of scope.
