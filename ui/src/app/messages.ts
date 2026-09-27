// Messages shown under the toolbar: errors (with the backend's report) and brief notices.
import { useCallback, useState } from "react";
import type { BackendError } from "../backend/commands";
import type { ErrorReport } from "../backend/generated/ErrorReport";

export interface MessageAction {
  label: string;
  run: () => void;
}

export type Message =
  | { id: number; kind: "error"; report: ErrorReport; action?: MessageAction }
  | { id: number; kind: "notice"; text: string; hint?: string };

/** How long a notice stays before it disappears on its own. Errors stay until dismissed. */
export const NOTICE_MS = 4000;

let nextId = 1;
const messageId = () => nextId++;

export function useMessages() {
  const [messages, setMessages] = useState<Message[]>([]);

  const dismiss = useCallback((id: number) => {
    setMessages((list) => list.filter((m) => m.id !== id));
  }, []);

  /** Shows an error, replacing any earlier error with the same code. */
  const showError = useCallback((error: BackendError, action?: MessageAction) => {
    const report = error.report;
    setMessages((list) => [
      ...list.filter((m) => m.kind !== "error" || m.report.code !== report.code),
      action
        ? { id: messageId(), kind: "error", report, action }
        : { id: messageId(), kind: "error", report },
    ]);
  }, []);

  const notice = useCallback(
    (text: string, hint?: string) => {
      const id = messageId();
      setMessages((list) => [
        ...list,
        hint === undefined ? { id, kind: "notice", text } : { id, kind: "notice", text, hint },
      ]);
      setTimeout(() => {
        dismiss(id);
      }, NOTICE_MS);
    },
    [dismiss],
  );

  return { messages, setMessages, dismiss, showError, notice };
}
