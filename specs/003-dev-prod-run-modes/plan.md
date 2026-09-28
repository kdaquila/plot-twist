# Implementation Plan: Dev and Prod Run Modes

**Branch**: `003-dev-prod-run-modes` | **Date**: 2026-09-27 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `specs/003-dev-prod-run-modes/spec.md`

## Summary

Dev mode (`npm run tauri dev`) and prod mode (`npm run tauri build`) already work; the error
contributors see comes from starting a development build without the Vite dev server
(`cargo run`), and a related case where another program holds port 5173 (research R1, R3).
Add a small dev-build-only startup check in `desktop/` that replaces those two browser pages
with a plain message naming `npm run tauri dev` (R2, R3). Rewrite the README's Development
section into "Dev mode", "Prod mode" and "Troubleshooting" (R4, R5). Verify both modes end to
end from a clean clone and record the steps (FR-009).

## Technical Context

**Language/Version**: Rust 1.98 (edition 2024); TypeScript 6 strict; Node 24

**Primary Dependencies**: Tauri 2.12 (`tauri::is_dev`, `WebviewWindow::navigate`, `Config::build.dev_url`), Vite 8; std `TcpStream` only — no new crates

**Storage**: N/A

**Testing**: `cargo test --workspace`, `npm run check`, perf tests, `scripts/check-startup.ps1`; manual end-to-end runs of both modes per quickstart.md

**Target Platform**: Windows x64

**Project Type**: desktop app (Tauri shell + Rust engine + local HTTP API + web frontend)

**Performance Goals**: unchanged budgets; startup measured on a prod build, which skips the check entirely

**Constraints**: the check runs only in development builds (`tauri::is_dev()`); no change to the documented dev command; no new dependencies

**Scale/Scope**: 1 new Rust file (~120 lines), 1 line in `desktop/src/startup/mod.rs`, README section rewrite

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Check | Status |
|---|---|---|
| I. Backend owns the data; every client equal | No data operation added. | Pass |
| II. Type safety | The check's three outcomes are an enum (`DevServer::{Ours, Missing, Other}`), not booleans or strings. | Pass |
| III. Validate with tests | The probe is validated by throwaway automated tests (fake servers on ephemeral ports: Ours/Other/Missing), purged before merge; both modes and the three dev-server situations are run end to end (quickstart). No kept test: the behavior is dev-tooling, not user acceptance of the product; a unit test of the HTML-title match would test an implementation detail. | Pass |
| IV. Performance budgets | Prod builds skip the check (`is_dev()` is false); startup budget re-measured. | Pass |
| V. Errors: closed vocabulary, no panics | No `unwrap`/`expect`; failures of the check are logged with `tracing` and fall back to leaving the page alone. The message is a developer-facing page, not an app/API error, so it is not an error-enum variant. | Pass |
| VI. Simplicity | One small module; no npm scripts or config added; commands documented rather than wrapped. | Pass |
| VII. Code Organization | New file in the existing `desktop/src/startup/` bucket; well under size limits. | Pass |
| Project constraints | Local API unchanged (localhost-only). Windows only. | Pass |

Post-design re-check: unchanged — Pass.

## Project Structure

### Documentation (this feature)

```text
specs/003-dev-prod-run-modes/
├── spec.md
├── plan.md
├── research.md
├── data-model.md
├── contracts/run-commands.md
├── quickstart.md
├── checklists/requirements.md
└── tasks.md            (/speckit-tasks)
```

### Source Code (touched)

```text
desktop/src/startup/
├── dev_server.rs       NEW: dev-build check of devUrl; message pages (R2, R3)
└── mod.rs              call dev_server::check(app) first in setup()
README.md               Development → Dev mode / Prod mode / Troubleshooting (R4, R5)
```

**Structure Decision**: Existing layout; one file added to the `startup` bucket.

## Implementation Approach

1. **Dev-server check** (`desktop/src/startup/dev_server.rs`): if `tauri::is_dev()` and
   `devUrl` is set, connect to each resolved address, send `GET /`, classify as
   `Ours` (reply contains `<title>plot-twist</title>`), `Missing` (no connection), or `Other`
   (connected, not ours). Probe only a loopback `devUrl` host. For `Missing`/`Other`, navigate `main` to a `data:` page with the
   matching message. A prototype of the `Missing` path exists in the working tree (R2).
2. **README**: replace the Development run steps with "Dev mode", "Prod mode" (installer and
   standalone paths, `--no-bundle` variant), and "Troubleshooting" (dev-server message, port in
   use, transient compile error, single instance). Keep the CI-checks list and Spec Kit notes.
3. **Verify** (quickstart.md): clean clone → dev mode (load, plot, script, HMR timing) →
   the three dev-server situations → prod standalone and installer (no dev server) → full CI
   sequence and budgets. Record results in `verification.md`.

## Complexity Tracking

No violations.
