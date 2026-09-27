import { open as openDialog } from "@tauri-apps/plugin-dialog";
import { useCallback, useEffect, useState } from "react";
import { frontendReady, toBackendError } from "../backend/commands";
import { DataPanel } from "../data-panel/DataPanel";
import { usePlotDraft } from "../data-panel/draft";
import { listenForDrops } from "../data-panel/drop";
import { PlotView } from "../plot/PlotView";
import { ErrorBoundary } from "./ErrorBoundary";
import { ErrorDisplay } from "./ErrorDisplay";
import { StatusBar } from "./StatusBar";
import { Toolbar } from "./Toolbar";
import { useAppliedTheme } from "./theme";
import { useSession } from "./useSession";
import "./theme.css";
import "./layout.css";

const FILE_FILTERS = [
  { name: "CSV files", extensions: ["csv", "tsv", "txt"] },
  { name: "All files", extensions: ["*"] },
];

export function App() {
  const session = useSession();
  const { state, settings, open, notice, showError } = session;
  const dark = useAppliedTheme(settings?.theme ?? "system");
  const { draft, change, shownPlot } = usePlotDraft(state.dataset, state.plot, session.applyPlot);
  const [dropping, setDropping] = useState(false);
  const [showFps, setShowFps] = useState(false);

  const openWithDialog = useCallback(() => {
    openDialog({ multiple: false, directory: false, filters: FILE_FILTERS }).then(
      (path) => {
        if (path !== null) open(path);
      },
      (e: unknown) => {
        showError(toBackendError(e));
      },
    );
  }, [open, showError]);

  // Tell the backend once the first frame is on screen (startup check, SC-003).
  useEffect(() => {
    const frame = requestAnimationFrame(() => {
      void frontendReady();
    });
    return () => {
      cancelAnimationFrame(frame);
    };
  }, []);

  useEffect(() => {
    const unlisten = listenForDrops({
      onFile: open,
      onMultiple: () => {
        notice("Drop a single file to open it.", "plot-twist opens one file at a time.");
      },
      onHover: setDropping,
    });
    return () => {
      void unlisten.then((fn) => {
        fn();
      });
    };
  }, [open, notice]);

  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (e.ctrlKey && !e.shiftKey && !e.altKey && e.key.toLowerCase() === "o") {
        e.preventDefault();
        openWithDialog();
      }
    };
    window.addEventListener("keydown", onKey);
    return () => {
      window.removeEventListener("keydown", onKey);
    };
  }, [openWithDialog]);

  return (
    <div className={dropping ? "app dropping" : "app"}>
      <Toolbar
        recentFiles={settings?.recent_files ?? []}
        theme={settings?.theme ?? "system"}
        showFps={showFps}
        onOpenDialog={openWithDialog}
        onOpenRecent={open}
        onTheme={session.setTheme}
        onShowFps={setShowFps}
      />
      <ErrorDisplay messages={session.messages} onDismiss={session.dismiss} />
      <DataPanel dataset={state.dataset} draft={draft} onDraft={change} onError={showError} />
      <main className="plot-area-wrapper">
        <ErrorBoundary area="plot">
          <PlotView
            dataset={state.dataset}
            plot={shownPlot}
            dark={dark}
            showFps={showFps}
            onError={showError}
          />
        </ErrorBoundary>
      </main>
      <StatusBar
        loading={state.loading}
        lastLoadMs={session.lastLoadMs}
        apiStatus={session.apiStatus}
      />
    </div>
  );
}
