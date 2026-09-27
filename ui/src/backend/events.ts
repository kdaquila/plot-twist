// Backend → GUI events (contracts/tauri-commands.md).
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { ApiStatus } from "./generated/ApiStatus";
import type { SessionEvent } from "./generated/SessionEvent";

export const onSessionChanged = (handler: (event: SessionEvent) => void): Promise<UnlistenFn> =>
  listen<SessionEvent>("session-changed", (e) => {
    handler(e.payload);
  });

export const onApiStatus = (handler: (status: ApiStatus) => void): Promise<UnlistenFn> =>
  listen<ApiStatus>("api-status", (e) => {
    handler(e.payload);
  });
