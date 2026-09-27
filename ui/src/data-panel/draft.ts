// The column selection being edited. Each complete selection (X and at least one Y) is sent
// to the backend with `set_plot`; the session's plot, from the window or a script, flows back.
import { useCallback, useState } from "react";
import type { DatasetSummary } from "../backend/generated/DatasetSummary";
import type { PlotConfig } from "../backend/generated/PlotConfig";
import type { PlotStyle } from "../backend/generated/PlotStyle";

export interface Draft {
  x: string | null;
  y: string[];
  style: PlotStyle;
}

const fromPlot = (plot: PlotConfig): Draft => ({ x: plot.x, y: plot.y, style: plot.style });

/** A new file starts with nothing selected (US1/AC8). */
const EMPTY: Draft = { x: null, y: [], style: "line" };

export function usePlotDraft(
  dataset: DatasetSummary | null,
  plot: PlotConfig | null,
  apply: (config: PlotConfig) => Promise<boolean>,
) {
  const [draft, setDraft] = useState<Draft>(() => (plot ? fromPlot(plot) : EMPTY));
  const [seen, setSeen] = useState({ dataset, plot });

  // Adopt a new dataset or a plot set elsewhere (e.g. by a script) during render.
  if (seen.dataset?.id !== dataset?.id || seen.plot !== plot) {
    setSeen({ dataset, plot });
    if (plot) setDraft(fromPlot(plot));
    else if (seen.dataset?.id !== dataset?.id) setDraft(EMPTY);
  }

  const change = useCallback(
    (next: Draft) => {
      setDraft(next);
      if (!dataset || next.x === null || next.y.length === 0) return;
      void apply({ dataset_id: dataset.id, x: next.x, y: next.y, style: next.style }).then((ok) => {
        if (!ok && plot) setDraft(fromPlot(plot));
      });
    },
    [apply, dataset, plot],
  );

  const complete = draft.x !== null && draft.y.length > 0;
  return { draft, change, shownPlot: complete ? plot : null };
}
