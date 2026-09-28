# Baseline: before the move (main @ 15699e6, measured 2026-09-27)

## Test counts (SC-004)

| Suite | Passed | Ignored |
|---|---|---|
| plot-twist (desktop) lib + bin unit | 0 | 0 |
| plot-twist-api unit | 0 | 0 |
| plot-twist-api `tests/acceptance.rs` | 6 | 0 |
| plot-twist-core unit | 21 | 0 |
| plot-twist-core `tests/import_malformed.rs` | 8 | 0 |
| plot-twist-core `tests/import_valid.rs` | 8 | 0 |
| plot-twist-core `tests/perf.rs` | 0 | 1 (run separately) |
| plot-twist-core `tests/view_faithfulness.rs` | 5 | 0 |
| **Rust total** | **48** | 1 |
| Vitest (3 files) | 15 | 0 |

## Performance (SC-005)

| Measure | Before | Budget |
|---|---|---|
| 1M-row load (95 MB) | 0.466 s | 3 s |
| View reduction | 14.4 ms | 50 ms |
| Memory | 0.84x file | 3x |
| Startup median (3 runs after warm-up) | 0.76 s | 2 s |

SC-005 compares against these same-machine numbers (±10%), which supersede feature 001's
(1.09 s / 0.51 s) as the reference because they were measured the same day on the same machine.

## Principle VII review after the move (T023)

- Tracked top-level folders: `api/ core/ desktop/ docs/ fixtures/ scripts/ specs/ ui/` (plus tool dot-folders).
  All role- or content-named; no `crates/`, `packages/`, `apps/`, `src-tauri/`, or `src/` at the root. Pass.
- Source roots: `core/src/`, `api/src/`, `desktop/src/`, `ui/src/`. Maximum folder depth below each:
  core 1, api 1, desktop 1, ui 3 (`ui/src/backend/generated/serde_json/`), all within the 3-level cap. Pass.

## After the move (T025)

| Check | Result |
|---|---|
| cargo fmt / clippy `-D warnings` | pass |
| Rust tests | 48 passed, 1 ignored (same as before) |
| `npm run check` (typecheck, lint, format, 15 Vitest tests, bindings drift) | pass |
| 1M-row load | 0.492 s (+5.5%, within 10%) |
| View reduction / memory | 13.8 ms / 0.84x (unchanged) |
| Startup median (`check-startup.ps1`, three sessions) | 0.98 s, 0.92 s, 1.02 s |

The startup runs were 20–30% above the morning baseline (0.76 s), so the pre-move `main` was
rebuilt in a separate worktree and both binaries were timed alternately, 10 rounds each:
**main 0.98 s median, branch 0.98 s median**. The difference from the morning figure is
machine state, not the move; against a same-session baseline the change is 0% (SC-005 pass).
All runs are well inside the 2 s budget.

## Clean-clone check (T021, SC-003)

Fresh `git worktree` of the branch: `npm ci`, `npm run build`, `npm run test`, `npm run dev`,
`npm run tauri build` all succeeded (installer `plot-twist_0.1.0_x64-setup.exe`, 2.81 MiB).
Afterwards: exactly one `node_modules/` (root), no `dist/`, no `ui/node_modules/`, frontend
build in `target/ui/`.
