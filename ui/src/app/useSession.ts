// Mirrors the backend session in React state. The backend is the source of truth: every
// change (from the window or from a script) arrives as a `session-changed` event.
import {
  useCallback,
  useEffect,
  useRef,
  useState,
  type Dispatch,
  type SetStateAction,
} from "react";
import * as backend from "../backend/commands";
import { BackendError } from "../backend/commands";
import { onApiStatus, onSessionChanged } from "../backend/events";
import type { ApiStatus } from "../backend/generated/ApiStatus";
import type { PlotConfig } from "../backend/generated/PlotConfig";
import type { SessionEvent } from "../backend/generated/SessionEvent";
import type { SessionState } from "../backend/generated/SessionState";
import type { Settings } from "../backend/generated/Settings";
import type { Theme } from "../backend/generated/Theme";
import { useMessages, type Message, type MessageAction } from "./messages";

const EMPTY: SessionState = { dataset: null, plot: null, loading: null };

export interface SessionModel {
  state: SessionState;
  settings: Settings | null;
  messages: Message[];
  lastLoadMs: number | null;
  apiStatus: ApiStatus | null;
  open: (path: string) => void;
  applyPlot: (config: PlotConfig) => Promise<boolean>;
  setTheme: (theme: Theme) => void;
  showError: (error: BackendError, action?: MessageAction) => void;
  notice: (text: string, hint?: string) => void;
  dismiss: (id: number) => void;
}

export function useSession(): SessionModel {
  const [state, setState] = useState<SessionState>(EMPTY);
  const [settings, setSettings] = useState<Settings | null>(null);
  const [lastLoadMs, setLastLoadMs] = useState<number | null>(null);
  const [apiStatus, setApiStatus] = useState<ApiStatus | null>(null);
  const loadStart = useRef<number | null>(null);
  const settingsRef = useRef<Settings | null>(null);
  useEffect(() => {
    settingsRef.current = settings;
  }, [settings]);

  const { messages, setMessages, dismiss, showError, notice } = useMessages();

  const refreshSettings = useCallback(() => {
    backend.getSettings().then(setSettings, (e: unknown) => {
      showError(backend.toBackendError(e));
    });
  }, [showError]);

  // Subscribe first, then read the current state, so no change is missed.
  useEffect(() => {
    let alive = true;
    const handle = (event: SessionEvent) => {
      if (!alive) return;
      applyEvent(event, {
        loadStart,
        setState,
        setLastLoadMs,
        clearErrors: () => {
          setMessages((list) => list.filter((m) => m.kind !== "error"));
        },
        refreshSettings,
        notice,
        showError,
      });
    };
    const unlisten = Promise.all([
      onSessionChanged(handle),
      onApiStatus((status) => {
        if (alive) setApiStatus(status);
      }),
    ]);
    void unlisten.then(async () => {
      try {
        const [current, stored, api] = await Promise.all([
          backend.getState(),
          backend.getSettings(),
          backend.getApiStatus(),
        ]);
        if (!alive) return;
        setState(current);
        setSettings(stored);
        setApiStatus((existing) => existing ?? api);
      } catch (e) {
        showError(backend.toBackendError(e));
      }
    });
    return () => {
      alive = false;
      void unlisten.then((fns) => {
        for (const fn of fns) fn();
      });
    };
  }, [notice, refreshSettings, setMessages, showError]);

  const open = useCallback(
    (path: string) => {
      backend.loadFile(path).catch((e: unknown) => {
        const error = backend.toBackendError(e);
        const recent = settingsRef.current?.recent_files.includes(path) ?? false;
        if (error.report.code === "FILE_NOT_FOUND" && recent) {
          showError(error, {
            label: "Remove from recent files",
            run: () => {
              backend.removeRecentFile(path).then(setSettings, (err: unknown) => {
                showError(backend.toBackendError(err));
              });
            },
          });
        } else {
          showError(error);
        }
      });
    },
    [showError],
  );

  const applyPlot = useCallback(
    async (config: PlotConfig) => {
      try {
        await backend.setPlot(config);
        return true;
      } catch (e) {
        showError(backend.toBackendError(e));
        return false;
      }
    },
    [showError],
  );

  const setTheme = useCallback(
    (theme: Theme) => {
      setSettings((s) => (s ? { ...s, theme } : s));
      backend.setTheme(theme).then(setSettings, (e: unknown) => {
        showError(backend.toBackendError(e));
      });
    },
    [showError],
  );

  return {
    state,
    settings,
    messages,
    lastLoadMs,
    apiStatus,
    open,
    applyPlot,
    setTheme,
    showError,
    notice,
    dismiss,
  };
}

interface EventContext {
  loadStart: { current: number | null };
  setState: Dispatch<SetStateAction<SessionState>>;
  setLastLoadMs: (ms: number) => void;
  clearErrors: () => void;
  refreshSettings: () => void;
  notice: (text: string) => void;
  showError: (error: BackendError) => void;
}

/** Applies one backend change to the mirrored state; script-initiated changes get a notice. */
function applyEvent(event: SessionEvent, ctx: EventContext) {
  switch (event.type) {
    case "load_started":
      ctx.loadStart.current = performance.now();
      ctx.setState((s) => ({ ...s, loading: { path: event.path } }));
      break;
    case "dataset_loaded":
      if (ctx.loadStart.current !== null)
        ctx.setLastLoadMs(performance.now() - ctx.loadStart.current);
      ctx.loadStart.current = null;
      ctx.setState({ dataset: event.dataset, plot: null, loading: null });
      // A new file clears the previous file's errors.
      ctx.clearErrors();
      ctx.refreshSettings();
      if (event.origin === "api") ctx.notice(`Loaded by script: ${event.dataset.file_name}`);
      break;
    case "load_failed":
      ctx.loadStart.current = null;
      ctx.setState((s) => ({ ...s, loading: null }));
      // Window-initiated failures are shown by the caller, which knows their context.
      if (event.origin === "api") ctx.showError(new BackendError(event.report));
      break;
    case "plot_changed":
      ctx.setState((s) => ({ ...s, plot: event.plot }));
      if (event.origin === "api") ctx.notice("Plot updated by script");
      break;
  }
}
