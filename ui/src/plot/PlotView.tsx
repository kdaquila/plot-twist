// The plot: WebGL canvas (PlotController) plus DOM overlays for axes, legend, hover
// readout, box-zoom rectangle, and empty states.
import { useEffect, useState } from "react";
import type { BackendError } from "../backend/commands";
import type { DatasetSummary } from "../backend/generated/DatasetSummary";
import type { PlotConfig } from "../backend/generated/PlotConfig";
import { Axis } from "./axes/Axis";
import { HoverTooltip } from "./HoverTooltip";
import { Legend } from "./legend/Legend";
import { usePlotController } from "./usePlotController";
import "./plot.css";

interface Props {
  dataset: DatasetSummary | null;
  plot: PlotConfig | null;
  dark: boolean;
  showFps: boolean;
  onError: (error: BackendError) => void;
}

export function PlotView({ dataset, plot, dark, showFps, onError }: Props) {
  const { areaRef, canvasHostRef, controllerRef, overlay, box, ready } = usePlotController(onError);
  const [hidden, setHidden] = useState<ReadonlySet<string>>(new Set());

  // A new dataset or X column shows every series again.
  const seriesKey = JSON.stringify([dataset?.id, plot?.x]);
  const [hiddenKey, setHiddenKey] = useState(seriesKey);
  if (hiddenKey !== seriesKey) {
    setHiddenKey(seriesKey);
    setHidden(new Set());
  }

  useEffect(() => {
    const gridColor = getComputedStyle(document.documentElement).getPropertyValue("--grid").trim();
    controllerRef.current?.update({ dataset, plot, hidden, dark, gridColor });
  }, [controllerRef, dataset, plot, hidden, dark, ready]);

  const controller = controllerRef.current;
  const xIsTime = dataset?.columns.find((c) => c.name === plot?.x)?.kind === "datetime";
  const allHidden = plot !== null && plot.y.every((y) => hidden.has(y));
  const empty = !dataset
    ? "Open a CSV file (File → Open) or drop one onto the window."
    : !plot
      ? "Choose an X column and at least one Y column."
      : allHidden
        ? "All series are hidden — click a legend entry to show it."
        : null;
  const hover = overlay.hover;

  return (
    <div className="plot">
      <Axis
        side="y"
        ticks={plot ? overlay.yTicks : []}
        title={plot?.y.length === 1 ? (plot.y[0] ?? null) : null}
      />
      <div
        ref={areaRef}
        className="plot-area"
        tabIndex={0}
        role="img"
        aria-label={
          plot
            ? `Plot of ${plot.y.join(", ")} against ${plot.x}. Drag to zoom, Shift+drag to pan, ` +
              "arrow keys pan, plus and minus zoom, 0 resets."
            : "Empty plot"
        }
      >
        <div ref={canvasHostRef} className="plot-canvas-host" />
        {box && (
          <div
            className="plot-box"
            style={{
              left: Math.min(box.x0, box.x1),
              top: Math.min(box.y0, box.y1),
              width: Math.abs(box.x1 - box.x0),
              height: Math.abs(box.y1 - box.y0),
            }}
          />
        )}
        {empty && <div className="plot-empty">{empty}</div>}
        {hover && !empty && plot && <HoverTooltip hit={hover} xName={plot.x} xIsTime={xIsTime} />}
        {plot && (
          <button
            className="plot-reset"
            onClick={() => controller?.reset()}
            onPointerDown={(e) => {
              e.stopPropagation();
            }}
            title="Reset view (double-click or 0)"
          >
            Reset view
          </button>
        )}
        {showFps && <div className="plot-fps">{overlay.fps} fps</div>}
      </div>
      <Axis side="x" ticks={plot ? overlay.xTicks : []} title={plot?.x ?? null} />
      {dataset && plot && (
        <Legend
          dataset={dataset}
          plot={plot}
          slotOf={(column) => controller?.colorSlot(column) ?? plot.y.indexOf(column)}
          hidden={hidden}
          dark={dark}
          onToggle={(column) => {
            setHidden((current) => {
              const next = new Set(current);
              if (!next.delete(column)) next.add(column);
              return next;
            });
          }}
        />
      )}
    </div>
  );
}
