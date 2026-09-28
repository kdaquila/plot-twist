# plot-twist Constitution

plot-twist is a free, open-source, modern data analysis GUI for researchers worldwide —
something analogous to ImageJ, but for datasets rather than images. It exists because the
field is split between expensive polished tools and powerful but dated ones.

## Core Principles

### I. Backend Owns the Data; Every Client Is Equal

- All data ingestion, parsing, transformation, statistics, and decimation MUST run in the
  Rust backend. The frontend MUST NOT hold full datasets; it receives only render-ready
  payloads (e.g., series decimated to the visible viewport).
- Every data operation the GUI can trigger MUST be exposed through the local API. The GUI,
  local scripts, and the MCP server (for BYOK AI agents) are all clients of that same API;
  there MUST be no GUI-only code path for a data operation. Pure view concerns (layout,
  hover, highlight, theme) are exempt.
- Work performed through the local API or MCP MUST be reflected in the running GUI so the
  user can inspect and continue working with the results.
- A feature is not complete until its data operations are reachable through the local API.

**Rationale**: Millions of points cannot live comfortably on the JavaScript side, and
automation (scripts, AI agents) is a first-class way to use the app. One backend API with
many equal clients keeps the GUI thin and guarantees anything a user can click, a script or
agent can do.

### II. Type Safety as a Quality Gate

Static typing is a primary quality bar, not a lint step run at the end.

- Rust: distinct concepts MUST use distinct types — newtype wrappers for ids and units
  (e.g., `DatasetId`, `ColumnId`), enums instead of booleans or magic strings, no
  stringly-typed data. Illegal states MUST be made unrepresentable rather than caught with
  runtime guards.
- TypeScript: `strict` mode MUST be enabled; `any` MUST NOT be used. Types describing the
  backend API MUST be generated from the Rust definitions, not hand-written, so the contract
  cannot drift.
- A red type check (`cargo check` or `tsc`) is treated as a failing test: it blocks the
  change from being considered done.

**Rationale**: Values that silently drift to the wrong shape — one kind of id passed where
another was expected, a unit or coordinate space confused for its neighbor — produce
plausible-looking but wrong results. For a research tool, a plausible wrong plot is the
worst possible failure, and the type system is the cheapest place to prevent it.

### III. Validate with Tests, Keep Only What Matters

- Every change MUST be validated by automated tests that its author (AI agent or human)
  runs before declaring the work done. Whether tests are written before or after the
  implementation is the author's choice.
- The tests kept in the repository MUST be limited to critical functionality whose failure
  would directly affect user acceptance. Tests of implementation details and tests written
  only to raise coverage MUST NOT be kept. There is no coverage target.
- Authors are encouraged to write as many throwaway tests as useful during implementation,
  and MUST purge them before merge, keeping only those that meet the bar above.

**Rationale**: Validation catches mistakes; a bloated suite of implementation-detail tests
resists refactoring and hides the tests that actually protect users. Separating
"tests used to verify the work" from "tests worth maintaining" gets the benefit of both.

### IV. Performance Budgets

The following budgets are non-negotiable for a dataset of 1M rows × 10 numeric columns:

- Pan and zoom MUST sustain 60 fps.
- Loading the dataset from CSV MUST complete in under 3 s.
- Cold start to an interactive window MUST complete in under 2 s.
- Resident memory MUST stay under 3× the size of the source file on disk.

Load, startup, and memory budgets MUST be covered by benchmarks that run in CI on the
Windows runner. Frame rate MUST be measured for any change touching rendering or the
decimation path, with the result recorded in the pull request. A change that pushes a
measurement past its budget MUST NOT merge.

**Rationale**: Researchers routinely have large files and modest hardware. Responsiveness
at scale is the main thing a modern tool offers over dated ones, and it only survives if
it is measured.

### V. Errors: Closed Vocabulary, No Panics

- Every error surfaced to a user or an API client MUST come from a closed, centralized set
  of Rust error enums — never an ad hoc string invented at the call site.
- Each error variant MUST map to a stable, machine-readable error code on the local API and
  MCP. Every code MUST have an actionable message and hint that say what failed, where
  (e.g., "row 1,204, column `temp`: not a number"), and what the user can do. This text
  is authored once, in the backend, so the GUI and API always show the same words; the GUI
  decides how to present each code (placement, follow-up actions).
- `unwrap`, `expect`, `panic!`, `todo!`, and `unreachable!` MUST NOT appear in non-test
  code; this is enforced with Clippy lints.
- The backend MUST emit structured logs to a file.

**Rationale**: The app ships to users who cannot read a stack trace, and to scripts and AI
agents that need to branch on failures programmatically. A closed vocabulary keeps error
reporting consistent, greppable, and testable, and keeps unrepresented failure states from
creeping back in through the error path (Principle II).

### VI. Simplicity (YAGNI)

Build for the current use case, not hypothetical future ones. No speculative abstractions,
plugin systems, or configuration surfaces for requirements that do not yet exist. Prefer
duplicating a few lines over introducing a shared abstraction with only one real caller.
Complexity must earn its place by solving a problem that exists today. The local API and
MCP surface required by Principle I are existing requirements, not speculation.

**Rationale**: A small, legible codebase is faster to develop and far easier for outside
contributors to join — which matters for an open-source project meant to outlive any one
maintainer's attention.

### VII. Code Organization

Structural discipline is enforced only through these rules.

**At the repository root**: every top-level folder is either a component named for its role
(`desktop/`, `core/`, `api/`, `ui/`) or a support area named for its contents (`docs/`,
`fixtures/`, `scripts/`, `specs/`). No top-level folder is named for a language, framework
template, or generic kind (`crates/`, `packages/`, `apps/`, `src-tauri/`, `src/`). A new
component gets its own role-named top-level folder.

**Within each component**, the source root is `<component>/src/`, and four rules apply
separately to each source root:

1. **Screaming architecture at the top level only.** The folders directly under a source
   root name the app's thematic buckets. Pick buckets that are undeniable and recognizable
   so they don't drift.
2. **Folder depth capped at 3 levels (4 at most)** below that root. Deeper nesting MUST
   justify itself rather than being the default.
3. **Functions/methods MUST stay under ~100 lines.**
4. **Files MUST stay under ~500 lines.** Inline Rust `#[cfg(test)]` modules do not count
   toward this limit.

No mandated `kind` subfolders (`utils/`, `types/`, etc.) and no one-export-per-file rule —
rely on IDE search and go-to-definition for anything finer-grained.

**Rationale**: A fixed folder grammar is ceremony maintained for its own sake. These rules
catch the actual failure modes — files and functions too large to hold in your head,
structure that doesn't communicate itself from the repository root down — without
prescribing an internal taxonomy.

## Project Constraints

- **License**: The project is free and open source under `GPL-3.0-or-later`.
- **Platform**: Windows (x64) is the only supported platform. Other platforms are out of
  scope unless added by amendment.
- **Stack**: The backend is Rust; the frontend is web technology written in TypeScript. The
  specific GUI shell and plotting libraries are chosen during planning, subject to these
  principles.
- **Dependencies**: Third-party dependencies may be added freely, subject to:
  - npm packages MUST be widely adopted (high download counts) and actively maintained;
    stay on the well-trodden path.
  - Rust crates are held to a looser bar: low release activity is acceptable when a crate
    is feature-complete or serves a niche need, given the smaller scientific ecosystem
    available without Python.
  - Every dependency's license MUST be compatible with GPL-3.0.
- **Local API and AI keys**: The local API and MCP server MUST bind to localhost only.
  Bring-your-own-key AI credentials are stored in a user configuration file outside the
  repository.

## Development Workflow

- All work happens on feature branches merged to `main` via pull request on GitHub.
- GitHub Actions CI on a Windows runner MUST run every quality gate, the kept test suite,
  and the performance benchmarks. Red CI blocks merge.
- Quality gates — each is treated the same as a failing test:
  - Rust: `cargo fmt --check` and `cargo clippy --all-targets -- -D warnings` pass. `unsafe`
    is permitted only with a `// SAFETY:` comment justifying it.
  - Frontend: `tsc` in strict mode, the formatter check, and the linter pass with zero
    warnings.
- A change is done only when: all quality gates pass, it has been validated by tests per
  Principle III (with throwaway tests purged), no performance budget is exceeded, and any
  new data operation is reachable through the local API.
- Contributions from outside collaborators are reviewed by a maintainer before merge. A
  maintainer's own changes MUST be self-checked against these principles, with CI as the
  objective backstop.

## Governance

This constitution supersedes ad hoc practice for this project. Amendments are made by
editing this file (via `/speckit-constitution` or equivalent) and MUST update the version
per semantic versioning:

- **MAJOR**: Backward-incompatible removal or redefinition of a principle.
- **MINOR**: A new principle or materially expanded guidance is added.
- **PATCH**: Wording, clarification, or non-semantic fixes.

Every amendment updates `Last Amended` below and produces a Sync Impact Report at the top of
this file for review; the report is removed before the amendment is committed. Specs, plans,
and tasks generated by Spec Kit commands MUST comply with these principles; a deviation MUST
be called out explicitly with its rationale rather than silently ignored.

**Version**: 1.1.0 | **Ratified**: 2026-09-27 | **Last Amended**: 2026-09-27
