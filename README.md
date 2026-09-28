# plot-twist

A free, open-source, modern data analysis GUI for researchers — something like
[ImageJ](https://imagej.net/), but for datasets rather than images.

The field is split between expensive, polished tools and powerful but dated ones. plot-twist
aims to put a fast, modern alternative into the hands of researchers worldwide, at no cost.

> **Status:** early development. There is no usable release yet.

## Goals

- **Large data, stays responsive.** Smooth 60 fps pan and zoom with millions of points.
- **Scriptable.** Everything you can do in the GUI is also available to your own scripts
  through a local API, and results show up in the GUI for you to explore.
- **AI-ready, on your terms.** An optional bring-your-own-key AI integration exposes the same
  local API to AI agents through a local MCP server.
- **Clear errors.** Failures say what went wrong, where, and what to do about it.

## Platform

Windows (x64) only.

## Architecture

A Rust backend owns all data loading and processing. A TypeScript web frontend renders
what the backend sends. The GUI, local scripts, and AI agents are all equal clients of the
same local API.

## Repository layout

| Folder | Contents |
|---|---|
| `desktop/` | The desktop app: window, startup, and the bridge between the frontend and the backend |
| `core/` | The data engine: CSV import, datasets, plot views, settings |
| `api/` | The local HTTP API that scripts and AI agents use |
| `ui/` | The frontend: toolbar, data panel, and the plot renderer |
| `docs/` | User and scripting documentation |
| `fixtures/` | Sample CSV files and example scripts used by tests and docs |
| `scripts/` | Contributor scripts, such as the startup-time check |
| `specs/` | Feature specifications, plans, and task lists |

Every component has the same shape, `<component>/src/<area>/`, for example
`core/src/csv_import/` and `ui/src/plot/`. Build output goes to `target/` (the frontend
build to `target/ui/`) and dependencies to the root `node_modules/`.

## Scripting

While the app is running, scripts can load files and set the plot through the local API.
See [docs/local-api.md](docs/local-api.md).

## Development

Prerequisites: [Rust](https://rustup.rs/) (the toolchain is pinned in `rust-toolchain.toml`),
Node.js 24, and the [Tauri prerequisites for Windows](https://tauri.app/start/prerequisites/)
(WebView2 is preinstalled on Windows 11).

Install the dependencies once, from the repository root:

```bash
npm ci
```

If your checkout predates the current layout, delete the leftover `dist/`, `ui/node_modules/`,
and `src-tauri/` folders; nothing uses them any more.

### Dev mode

Use dev mode while working on the app:

```bash
npm run tauri dev
```

This starts the frontend dev server and opens the app. Changes you save in `ui/` show up in
the window straight away. Changes in `desktop/`, `api/`, or `core/` rebuild and restart the
app. Press Ctrl+C in the terminal to stop.

### Prod mode

Use prod mode to get the app as a user would: optimized, self-contained, and without a dev
server.

```bash
npm run tauri build
```

This writes two things:

- the installer: `target/release/bundle/nsis/plot-twist_<version>_x64-setup.exe`
- the standalone app: `target/release/plot-twist.exe`, which you can run directly

To build only the standalone app, which is quicker:

```bash
npm run tauri build -- --no-bundle
```

### Troubleshooting

- **"The dev server is not running"** (or, on older checkouts, "Hmmm… can't reach this
  page"): the app was started with `cargo run`, which does not start the frontend dev server.
  Close the window and use `npm run tauri dev`.
- **"Port 5173 is already in use"**: another program is using the dev server's port. Close
  that program, then run `npm run tauri dev` again.
- **A compile error while you edit**: dev mode rebuilds on every save, so a half-finished
  edit can fail to compile. Keep editing; the next save that compiles rebuilds and restarts
  the app.
- **Nothing seems to happen when the app starts**: plot-twist runs one copy at a time, and a
  copy that is already running (from dev mode, prod mode, or an installed copy) comes to the
  front instead. Close it first.

Before opening a pull request, run the same checks as CI:

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

The performance test (1 million rows × 10 columns) runs in release mode:

```bash
cargo test -p plot-twist-core --release --test perf -- --ignored --nocapture
```

This project is developed spec-first with [Spec Kit](https://github.com/github/spec-kit).
The project's principles, quality gates, and workflow are defined in the
[constitution](.specify/memory/constitution.md).

The Spec Kit agent commands are generated per machine and are not committed. After cloning,
install the [Specify CLI](https://github.com/github/spec-kit) and run:

```bash
specify init --here --integration claude --script py
```

If it modifies any tracked files under `.specify/`, restore them with
`git restore .specify` so the project's constitution and templates are kept.

## License

Copyright (C) 2026 Kenneth D'Aquila

This program is free software: you can redistribute it and/or modify it under the terms of
the GNU General Public License as published by the Free Software Foundation, either version
3 of the License, or (at your option) any later version.

This program is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY;
without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.
See the [GNU General Public License](LICENSE) for more details.

SPDX-License-Identifier: `GPL-3.0-or-later`
