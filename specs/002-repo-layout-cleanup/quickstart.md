# Quickstart: Validate the Repository Layout Cleanup

Run from the repository root on branch `002-repo-layout-cleanup`.

## 1. Layout (US1, SC-001, SC-002)

```bash
ls -d */
```

Expect: `api/ core/ desktop/ docs/ fixtures/ scripts/ specs/ ui/` plus gitignored
`node_modules/ target/`. No `crates/`, `src-tauri/`, or `dist/`.

Every thematic bucket sits at depth 3: `core/src/csv_import/`, `api/src/routes/`,
`desktop/src/startup/`, `ui/src/plot/`.

## 2. Build, check, test (US2, SC-004)

```bash
npm ci
```

```bash
npm run build
```

```bash
cargo fmt --all --check
```

```bash
cargo clippy --workspace --all-targets -- -D warnings
```

```bash
cargo test --workspace
```

```bash
npm run check
```

Expect: all pass; test counts equal the pre-move counts recorded in tasks.md.

## 3. No stray folders (US3, SC-003)

After step 2 plus `npm run dev` (stop it) and `npm run tauri build`:

```bash
find . -name node_modules -type d -prune -not -path './target/*'
```

Expect exactly `./node_modules`. `target/ui/index.html` exists; no `dist/` at the root.

## 4. Performance (SC-005)

```bash
cargo test -p plot-twist-core --release --test perf -- --ignored --nocapture
```

```bash
pwsh ./scripts/check-startup.ps1
```

Expect: within 10% of 0.51 s load and 1.09 s startup median.

## 5. Behavior (FR-012)

`npm run tauri dev`, open `fixtures/csv/sensors.csv`, plot two Y columns; then run
`python fixtures/scripts/load_and_plot.py` against the open app. Expect identical behavior to
feature 001.

## 6. History (SC-006)

```bash
git log --follow --oneline -- core/src/lib.rs
```

Expect commits from before the move.

## 7. Constitution (US4)

Principle VII includes the repository-root rule; version reads 1.1.0.
