// Orchestrates one plot: view range, backend view data, GPU meshes, and overlay state.
// Framework-free so React re-renders never touch the render loop.
import type { BackendError } from "../backend/commands";
import type { DatasetSummary } from "../backend/generated/DatasetSummary";
import type { PlotConfig } from "../backend/generated/PlotConfig";
import type { ViewRequest } from "../backend/generated/ViewRequest";
import type { SeriesPayload } from "../backend/viewPayload";
import { numericTicks, timeTicks, type Tick } from "./axes/format";
import { nearestPoint, type HoverHit } from "./hover";
import { assignSlots, seriesStyle } from "./legend/palette";
import { PlotRenderer } from "./renderer/PlotRenderer";
import { ViewFetcher } from "./viewData";
import * as view from "./viewState";

export interface PositionedTick extends Tick {
  px: number;
}

export interface Overlay {
  range: view.Range | null;
  xTicks: PositionedTick[];
  yTicks: PositionedTick[];
  hover: HoverHit | null;
  fps: number;
}

export interface PlotInputs {
  dataset: DatasetSummary | null;
  plot: PlotConfig | null;
  hidden: ReadonlySet<string>;
  dark: boolean;
  gridColor: string;
}

interface Payload {
  series: SeriesPayload[];
  origin: { x: number; y: number };
  style: "line" | "scatter";
}

const MAX_PIXELS = 8192;

export class PlotController {
  private readonly renderer = new PlotRenderer();
  private readonly fetcher: ViewFetcher;
  private inputs: PlotInputs = {
    dataset: null,
    plot: null,
    hidden: new Set(),
    dark: false,
    gridColor: "#e5e5e5",
  };
  private slots = new Map<string, number>();
  private range: view.Range | null = null;
  private full: view.Range | null = null;
  private refitY = false;
  private payload: Payload | null = null;
  private width = 1;
  private height = 1;
  private pointer: { px: number; py: number } | null = null;
  private frame = 0;
  private frameTimes: number[] = [];

  constructor(
    private readonly onOverlay: (overlay: Overlay) => void,
    onError: (error: BackendError) => void,
  ) {
    this.fetcher = new ViewFetcher(
      () => this.buildRequest(),
      (request, series) => {
        this.receive(request, series);
      },
      (error) => {
        // A dataset or plot replaced mid-request is expected; anything else is shown.
        if (!["STALE_DATASET", "NO_PLOT", "UNKNOWN_COLUMN"].includes(error.report.code)) {
          onError(error);
        }
      },
    );
  }

  init(host: HTMLElement): Promise<boolean> {
    this.renderer.onRestored = () => {
      this.requestRender();
    };
    return this.renderer.init(host);
  }

  destroy() {
    this.fetcher.stop();
    cancelAnimationFrame(this.frame);
    this.renderer.destroy();
  }

  resize(width: number, height: number) {
    this.width = Math.max(1, width);
    this.height = Math.max(1, height);
    this.renderer.resize(this.width, this.height);
    this.fetcher.request();
    this.requestRender();
  }

  /** Applies new inputs and the view policy (FR-009d, FR-009d2). */
  update(next: PlotInputs) {
    const previous = this.inputs;
    this.inputs = next;
    const { dataset, plot } = next;
    if (!dataset || !plot) {
      this.range = null;
      this.payload = null;
      this.renderer.clearSeries();
      this.requestRender();
      return;
    }
    this.slots = assignSlots(this.slots, plot.y);
    this.full = this.fullRange();
    const datasetChanged = previous.dataset?.id !== dataset.id;
    const plotChanged = datasetChanged || !samePlot(previous.plot, plot);
    if (plotChanged) {
      const xChanged = datasetChanged || previous.plot?.x !== plot.x;
      if (xChanged || !this.range) {
        this.range = this.full;
        this.refitY = false;
        this.payload = null;
        this.renderer.clearSeries();
      } else {
        this.refitY = true;
      }
      this.fetcher.request();
    } else if (previous.dark !== next.dark || previous.hidden !== next.hidden) {
      this.rebuildMeshes();
    }
    this.requestRender();
  }

  get dataset() {
    return this.inputs.dataset;
  }

  get xIsTime(): boolean {
    const { dataset, plot } = this.inputs;
    return dataset?.columns.find((c) => c.name === plot?.x)?.kind === "datetime";
  }

  colorSlot(column: string): number | undefined {
    return this.slots.get(column);
  }

  // ---- interactions (all in CSS pixels relative to the plot area) ----

  reset() {
    this.full = this.fullRange();
    if (!this.full) return;
    this.setRange(this.full);
  }

  panPx(dxPx: number, dyPx: number) {
    if (!this.range) return;
    const r = this.range;
    const dx = (-dxPx / this.width) * (r.xMax - r.xMin);
    const dy = (dyPx / this.height) * (r.yMax - r.yMin);
    this.setRange(view.pan(r, dx, dy));
  }

  zoomAtPx(px: number, py: number, factor: number) {
    if (!this.range || !this.full) return;
    const [x, y] = this.toData(px, py);
    this.setRange(view.zoomAt(this.range, x, y, factor, view.limitsFor(this.full)));
  }

  zoomRectPx(x0: number, y0: number, x1: number, y1: number) {
    if (!this.range || !this.full) return;
    const [ax, ay] = this.toData(Math.min(x0, x1), Math.max(y0, y1));
    const [bx, by] = this.toData(Math.max(x0, x1), Math.min(y0, y1));
    const r = this.range;
    // A drag that is thin in one direction only zooms the other axis.
    const rect = {
      xMin: Math.abs(x1 - x0) < 5 ? r.xMin : ax,
      xMax: Math.abs(x1 - x0) < 5 ? r.xMax : bx,
      yMin: Math.abs(y1 - y0) < 5 ? r.yMin : ay,
      yMax: Math.abs(y1 - y0) < 5 ? r.yMax : by,
    };
    this.setRange(view.zoomToRect(rect, view.limitsFor(this.full)));
  }

  hover(px: number, py: number) {
    this.pointer = { px, py };
    this.emitOverlay();
  }

  clearHover() {
    this.pointer = null;
    this.emitOverlay();
  }

  // ---- internals ----

  private toData(px: number, py: number): [number, number] {
    const r = this.range ?? { xMin: 0, xMax: 1, yMin: 0, yMax: 1 };
    return [
      r.xMin + (px / this.width) * (r.xMax - r.xMin),
      r.yMax - (py / this.height) * (r.yMax - r.yMin),
    ];
  }

  private setRange(range: view.Range) {
    this.range = range;
    this.fetcher.request();
    this.requestRender();
  }

  private visibleY(): string[] {
    return (this.inputs.plot?.y ?? []).filter((name) => !this.inputs.hidden.has(name));
  }

  private fullRange(): view.Range | null {
    const { dataset, plot } = this.inputs;
    if (!dataset || !plot) return null;
    const extent = (name: string): view.Extent | null => {
      const c = dataset.columns.find((col) => col.name === name);
      return c?.min != null && c.max != null ? { min: c.min, max: c.max } : null;
    };
    return view.fit(extent(plot.x), view.union(this.visibleY().map(extent)));
  }

  private buildRequest(): ViewRequest | null {
    const { dataset, plot } = this.inputs;
    if (!dataset || !plot || !this.range) return null;
    const clampPx = (v: number) => Math.min(MAX_PIXELS, Math.max(1, Math.round(v)));
    return {
      dataset_id: dataset.id,
      x_min: this.range.xMin,
      x_max: this.range.xMax,
      y_min: this.range.yMin,
      y_max: this.range.yMax,
      width_px: clampPx(this.width),
      height_px: clampPx(this.height),
    };
  }

  private receive(request: ViewRequest, series: SeriesPayload[]) {
    const plot = this.inputs.plot;
    if (!plot || request.dataset_id !== this.inputs.dataset?.id) return;
    this.payload = {
      series,
      origin: { x: (request.x_min + request.x_max) / 2, y: (request.y_min + request.y_max) / 2 },
      style: plot.style,
    };
    if (this.refitY && this.range) {
      this.refitY = false;
      const visible = new Set(this.visibleY());
      const extents = series
        .filter((s) => visible.has(this.columnName(s.columnIndex)))
        .map((s) => (s.yExtent ? { min: s.yExtent[0], max: s.yExtent[1] } : null));
      const [yMin, yMax] = view.padded(view.union(extents));
      this.range = { ...this.range, yMin, yMax };
      this.fetcher.request();
    }
    this.rebuildMeshes();
    this.requestRender();
  }

  private columnName(index: number): string {
    return this.inputs.dataset?.columns[index]?.name ?? "";
  }

  private rebuildMeshes() {
    if (!this.payload) return;
    const { series, origin, style } = this.payload;
    this.renderer.setSeries(
      series.map((s) => {
        const name = this.columnName(s.columnIndex);
        return {
          key: name,
          points: s.points,
          look: seriesStyle(this.slots.get(name) ?? 0, this.inputs.dark),
          visible: !this.inputs.hidden.has(name),
        };
      }),
      origin,
      style,
    );
    this.emitOverlay();
  }

  private requestRender() {
    if (this.frame) return;
    this.frame = requestAnimationFrame(() => {
      this.frame = 0;
      this.render();
    });
  }

  private render() {
    const now = performance.now();
    this.frameTimes.push(now);
    while ((this.frameTimes[0] ?? now) < now - 1000) this.frameTimes.shift();
    const overlay = this.overlay();
    if (this.range) {
      this.renderer.render(this.range, {
        xPx: overlay.xTicks.map((t) => t.px),
        yPx: overlay.yTicks.map((t) => t.px),
        color: this.inputs.gridColor,
      });
    } else {
      this.renderer.render({ xMin: 0, xMax: 1, yMin: 0, yMax: 1 }, { xPx: [], yPx: [], color: "" });
    }
    this.onOverlay(overlay);
  }

  private emitOverlay() {
    this.onOverlay(this.overlay());
  }

  private overlay(): Overlay {
    const r = this.range;
    if (!r) return { range: null, xTicks: [], yTicks: [], hover: null, fps: 0 };
    const xTicks = (this.xIsTime ? timeTicks : numericTicks)(r.xMin, r.xMax, this.width)
      .map((t) => ({ ...t, px: ((t.value - r.xMin) / (r.xMax - r.xMin)) * this.width }))
      .filter((t) => t.px >= 0 && t.px <= this.width);
    const yTicks = numericTicks(r.yMin, r.yMax, this.height)
      .map((t) => ({ ...t, px: ((r.yMax - t.value) / (r.yMax - r.yMin)) * this.height }))
      .filter((t) => t.px >= 0 && t.px <= this.height);
    return { range: r, xTicks, yTicks, hover: this.hoverHit(r), fps: this.frameTimes.length };
  }

  private hoverHit(range: view.Range): HoverHit | null {
    if (!this.pointer || !this.payload) return null;
    const visible = new Set(this.visibleY());
    const series = this.payload.series
      .map((s) => ({ name: this.columnName(s.columnIndex), points: s.points }))
      .filter((s) => visible.has(s.name));
    return nearestPoint(series, range, this.width, this.height, this.pointer.px, this.pointer.py);
  }
}

function samePlot(a: PlotConfig | null, b: PlotConfig | null): boolean {
  return (
    a === b ||
    (a !== null &&
      b !== null &&
      a.dataset_id === b.dataset_id &&
      a.x === b.x &&
      a.style === b.style &&
      a.y.length === b.y.length &&
      a.y.every((y, i) => y === b.y[i]))
  );
}
