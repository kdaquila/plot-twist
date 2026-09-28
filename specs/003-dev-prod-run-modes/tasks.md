# Tasks: Dev and Prod Run Modes

**Input**: Design documents from `specs/003-dev-prod-run-modes/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/run-commands.md, quickstart.md

**Tests**: No kept tests (plan, Constitution III); T017 validates the probe with throwaway automated tests that are purged before merge. Behavior is validated by the runs in quickstart.md; existing suites are the regression net (SC-005).

## Format: `[ID] [P?] [Story] Description`

## Phase 1: Setup

- [X] T001 Record the baseline on `main` (before this feature's changes) in specs/003-dev-prod-run-modes/verification.md: in a `git worktree` of `main`, per-suite pass counts from `cargo test --workspace` and `npm run test`, perf numbers from `cargo test -p plot-twist-core --release --test perf -- --ignored --nocapture`, and the startup median from `scripts/check-startup.ps1` (SC-005)

---

## Phase 2: Foundational

No shared prerequisites beyond Phase 1: US1's code change and US2's verification are independent.

---

## Phase 3: User Story 1 - Run the app in dev mode without errors (Priority: P1)

**Goal**: A development build never shows a browser error page; it shows a message naming `npm run tauri dev` (FR-003, FR-008; research R2, R3).
**Independent Test**: quickstart.md §2 and §3.

- [X] T002 [US1] Finish desktop/src/startup/dev_server.rs from the prototype: add `enum DevServer { Ours, Missing, Other }`; `probe(&Url) -> DevServer` connects to each address `(host, port).to_socket_addrs()` yields (300 ms timeout), sends `GET / HTTP/1.1` with `Host` and `Connection: close`, reads the reply with a read timeout, and returns `Ours` if it contains `<title>plot-twist</title>`, `Other` if connected but not ours, `Missing` if no address connects; `check(app)` returns immediately unless `tauri::is_dev()` and `build.dev_url` is set, then for `Missing`/`Other` navigates window `main` to a `data:` page. Two messages: Missing → heading "The dev server is not running", body naming `npm run tauri dev`; Other → heading "Port <port> is used by another program", body saying to close that program and run `npm run tauri dev`. Only probe when the `devUrl` host is `localhost`, `127.0.0.1` or `::1` (otherwise leave the page alone). Log each outcome with `tracing`; no `unwrap`/`expect`; file < 500 lines, functions < 100 lines (Constitution II, V, VII)
- [X] T003 [US1] Confirm `dev_server::check(app)` is the first call after the "starting" log in `setup()` in desktop/src/startup/mod.rs (added during research) and that `mod dev_server;` is declared
- [X] T004 [US1] Run `cargo fmt --all --check` and `cargo clippy --workspace --all-targets -- -D warnings`; fix any findings in desktop/src/startup/dev_server.rs
- [X] T017 [US1] Validate `probe` with throwaway automated tests in a `#[cfg(test)]` module of desktop/src/startup/dev_server.rs: a `TcpListener` on an ephemeral localhost port replying with a page containing `<title>plot-twist</title>` → `Ours`; replying with other HTML → `Other`; a port with nothing listening → `Missing`. Run `cargo test -p plot-twist`, then delete the module before commit (dev tooling, not user acceptance) (Constitution III)
- [X] T005 [US1] With no dev server running, `cargo build -p plot-twist` then start target/debug/plot-twist.exe; screenshot; confirm the "dev server is not running" page appears within 5 s and the local API still starts (api.json written under %LOCALAPPDATA%\plot-twist) (FR-003, SC-003)
- [X] T006 [US1] Hold port 5173 with `node -e "require('http').createServer((q,r)=>r.end('other')).listen(5173,'localhost')"`; start target/debug/plot-twist.exe and confirm the "Port 5173 is used by another program" page; then run `npm run tauri dev` and confirm Vite's "Port 5173 is already in use", exit code 1, and that any window left open shows the port page; stop everything (FR-008)
- [X] T007 [US1] Run `npm run tauri dev`: confirm the normal app appears (check returns `Ours`), load fixtures/csv/sensors.csv, plot two Y columns, run `python fixtures/scripts/load_and_plot.py`; edit a visible string in ui/src/, time the update (≤ 2 s, no restart), revert it; touch a file in core/src/ and confirm rebuild and relaunch (US1 AC1–AC3, SC-004)

**Checkpoint**: Dev mode works and every way of starting a development build explains itself.

---

## Phase 4: User Story 2 - Build and run the app in prod mode (Priority: P1)

**Goal**: The prod build works standalone and installed, with no dev server (FR-004, FR-005; research R4).
**Independent Test**: quickstart.md §4.

- [X] T008 [US2] With no dev server running, run `npm run tauri build`; confirm target/release/plot-twist.exe and target/release/bundle/nsis/plot-twist_0.1.0_x64-setup.exe exist; start the exe and repeat T007's load, plot, and script check; confirm the dev-server check is skipped (no dev-server log line in %LOCALAPPDATA%\plot-twist\logs) (US2 AC1–AC2, FR-005)
- [X] T009 [US2] Run the installer, start the installed plot-twist from the Start menu, repeat the load, plot, and script check, then uninstall and confirm it is gone (US2 AC3)
- [X] T010 [US2] Delete target/release/bundle/, run `npm run tauri build -- --no-bundle`, and confirm only target/release/plot-twist.exe is produced (no bundle/ folder) (US2 AC4, FR-004)

**Checkpoint**: Prod mode builds and runs from both deliverables.

---

## Phase 5: User Story 3 - The README explains both modes (Priority: P2)

**Goal**: One README section tells a contributor how to run each mode and what to do when something goes wrong (FR-006, FR-007).
**Independent Test**: A reader can do quickstart.md §2–§4 from the README alone.

- [X] T011 [US3] In README.md, replace the run steps in "Development" with: a prerequisites line (kept), `npm ci`, then "### Dev mode" (`npm run tauri dev`; for working on the app; hot reload of ui/, rebuild-and-restart on desktop/, api/, core/; Ctrl+C stops it), and "### Prod mode" (`npm run tauri build` → installer at `target/release/bundle/nsis/` and standalone `target/release/plot-twist.exe`; `npm run tauri build -- --no-bundle` for the standalone app only; runs without the checkout or a dev server). Keep the old-checkout cleanup note, the CI-checks list, the perf test, and the Spec Kit notes unchanged (FR-006)
- [X] T012 [US3] Add "### Troubleshooting" to README.md after Prod mode, one short entry each with cause and fix: "The dev server is not running" page (or "can't reach this page" on older checkouts) — started with `cargo run`; use `npm run tauri dev`; "Port 5173 is already in use" — another program holds it; close it and retry; a compile error while editing — the watcher keeps running and the next good save rebuilds; nothing happens on launch — a copy (dev or installed) is already running and was brought to the front; close it first (FR-007)
- [X] T013 [US3] Run `npm run format:check` and fix README.md formatting if needed

**Checkpoint**: README covers both modes and all four troubleshooting cases.

---

## Phase 6: Polish & Cross-Cutting

- [ ] T014 In a fresh `git worktree add ../pt-verify 003-dev-prod-run-modes` (clean clone: no node_modules/, no target/), follow only the README: `npm ci`, dev mode (load, plot, script), prod mode (`npm run tauri build`, run the exe with no dev server); record each step and result in specs/003-dev-prod-run-modes/verification.md; remove the worktree afterwards (FR-009, SC-001, SC-002)
- [ ] T015 Run the full CI sequence from .github/workflows/ci.yml locally (npm run build, cargo fmt, clippy, cargo test, npm run check, perf test, scripts/check-startup.ps1); record counts and numbers in verification.md and confirm test counts equal T001 and perf/startup are within 10% (FR-010, SC-005)
- [X] T016 Search README.md and docs/ for other instructions to start the app (e.g. `cargo run`, `tauri dev`) and align them with the new sections

---

## Dependencies & Execution Order

- T001 first (baseline must predate verification; it runs in a separate `main` worktree, so the prototype in this working tree does not affect it).
- US1: T002 → T003 → T004 → T017 → T005, T006, T007.
- US2 (T008–T010) is independent of US1's code (prod builds skip the check) but runs after T004 so the built exe includes the final code.
- US3 (T011–T013) after US1 and US2 so the README documents verified behavior.
- Polish (T014–T016) last.

## Parallel Example

```text
T011 README dev/prod sections  |  T008 prod build verification (different files; T011 is docs-only)
```

## Implementation Strategy

MVP = Phase 1 + US1: the reported error is fixed and dev mode verified. Then US2 verifies prod
mode, US3 documents both, and Phase 6 proves it all from a clean clone.
