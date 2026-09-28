# Quickstart: Validate Dev and Prod Run Modes

Run from the repository root on branch `003-dev-prod-run-modes`. Close any running plot-twist
first (single instance). Record results in `verification.md`.

## 1. Clean clone (FR-009, SC-001, SC-002)

```bash
git worktree add ../pt-verify 003-dev-prod-run-modes
```

In `../pt-verify`, follow only the README from here on.

## 2. Dev mode (US1, SC-001, SC-004)

```bash
npm ci
```

```bash
npm run tauri dev
```

Expect the app window. Load `fixtures/csv/sensors.csv`, plot two Y columns, run
`python fixtures/scripts/load_and_plot.py`. Edit a visible string in `ui/src/`, save, and time
until it shows (≤ 2 s, no restart). Touch a file in `core/src/` and confirm rebuild + relaunch.

## 3. Dev-server situations (FR-003, FR-008, SC-003)

With Vite stopped:

```bash
cargo run
```

Expect "The dev server is not running" naming `npm run tauri dev`, within 5 s. Then hold the
port with another program and repeat:

```bash
node -e "require('http').createServer((q,r)=>r.end('other')).listen(5173,'localhost')"
```

Expect `cargo run` → "Port 5173 is used by another program"; `npm run tauri dev` → Vite's
"Port 5173 is already in use", exit 1, and any window shows the port message.

## 4. Prod mode (US2, SC-002)

```bash
npm run tauri build
```

With no dev server running, start `target/release/plot-twist.exe`; repeat step 2's load, plot,
and script check. Run the installer from `target/release/bundle/nsis/`, start the installed app,
repeat, then uninstall. Confirm `npm run tauri build -- --no-bundle` builds only the exe.

## 5. Checks and budgets (FR-010, SC-005)

The CI sequence from `.github/workflows/ci.yml`: build, fmt, clippy, test, check, perf,
`scripts/check-startup.ps1`. Test counts equal the pre-change baseline; perf and startup within
10%.
