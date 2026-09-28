// Typed wrappers over the Tauri commands (contracts/tauri-commands.md). Types are generated
// from Rust; never hand-write backend types here.
import { invoke } from "@tauri-apps/api/core";
import type { ApiStatus } from "./generated/ApiStatus";
import type { BadCellPage } from "./generated/BadCellPage";
import type { DatasetId } from "./generated/DatasetId";
import type { ErrorReport } from "./generated/ErrorReport";
import type { LoadResult } from "./generated/LoadResult";
import type { PlotConfig } from "./generated/PlotConfig";
import type { SessionState } from "./generated/SessionState";
import type { Settings } from "./generated/Settings";
import type { Theme } from "./generated/Theme";
import type { ViewRequest } from "./generated/ViewRequest";

/** A failure reported by the backend, carrying its `ErrorReport`. */
export class BackendError extends Error {
  constructor(readonly report: ErrorReport) {
    super(report.message);
    this.name = "BackendError";
  }
}

function isErrorReport(value: unknown): value is ErrorReport {
  return (
    typeof value === "object" &&
    value !== null &&
    "code" in value &&
    "message" in value &&
    "hint" in value
  );
}

/** Normalizes anything thrown by a command into a `BackendError`. */
export function toBackendError(error: unknown): BackendError {
  if (error instanceof BackendError) return error;
  if (isErrorReport(error)) return new BackendError(error);
  return new BackendError({
    code: "INTERNAL",
    message: "Something went wrong talking to the plot-twist backend.",
    hint: "Please report this issue.",
    line: null,
    column: null,
    value: null,
    details: { reason: String(error) },
  });
}

async function call<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  try {
    return await invoke<T>(command, args);
  } catch (error) {
    throw toBackendError(error);
  }
}

export const loadFile = (path: string) => call<LoadResult>("load_file", { path });
export const getState = () => call<SessionState>("get_state");
export const setPlot = (config: PlotConfig) => call<PlotConfig>("set_plot", { config });
export const getBadCells = (datasetId: DatasetId, offset: number, limit: number) =>
  call<BadCellPage>("get_bad_cells", { datasetId, offset, limit });
export const getView = (request: ViewRequest) => call<ArrayBuffer>("get_view", { request });
export const getSettings = () => call<Settings>("get_settings");
export const setTheme = (theme: Theme) => call<Settings>("set_theme", { theme });
export const removeRecentFile = (path: string) => call<Settings>("remove_recent_file", { path });
export const getApiStatus = () => call<ApiStatus | null>("get_api_status");
export const frontendReady = () => call<null>("frontend_ready");
