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

## Development

This project is developed spec-first with [Spec Kit](https://github.com/github/spec-kit).
The project's principles, quality gates, and workflow are defined in the
[constitution](.specify/memory/constitution.md).

## License

Copyright (C) 2026 Kenneth D'Aquila

This program is free software: you can redistribute it and/or modify it under the terms of
the GNU General Public License as published by the Free Software Foundation, either version
3 of the License, or (at your option) any later version.

This program is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY;
without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.
See the [GNU General Public License](LICENSE) for more details.

SPDX-License-Identifier: `GPL-3.0-or-later`
