// Drag-and-drop opening (FR-001a): exactly one file loads; more than one loads nothing.
import type { UnlistenFn } from "@tauri-apps/api/event";
import { getCurrentWebview } from "@tauri-apps/api/webview";

export interface DropHandlers {
  onFile: (path: string) => void;
  onMultiple: () => void;
  onHover: (over: boolean) => void;
}

export function listenForDrops(handlers: DropHandlers): Promise<UnlistenFn> {
  return getCurrentWebview().onDragDropEvent(({ payload }) => {
    switch (payload.type) {
      case "enter":
      case "over":
        handlers.onHover(true);
        break;
      case "leave":
        handlers.onHover(false);
        break;
      case "drop": {
        handlers.onHover(false);
        const [first, ...rest] = payload.paths;
        if (first !== undefined && rest.length === 0) handlers.onFile(first);
        else if (rest.length > 0) handlers.onMultiple();
        break;
      }
    }
  });
}
