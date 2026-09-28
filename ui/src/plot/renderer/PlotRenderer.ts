// Owns the PixiJS application: gridlines plus one mesh per series. Renders on demand only.
// The app CSP forbids eval; this swaps Pixi's generated code for static equivalents.
import "pixi.js/unsafe-eval";
import { Application, Container, Graphics } from "pixi.js";
import type { Range } from "../viewState";
import { SeriesMesh, type Origin, type SeriesLook } from "./seriesMesh";

export interface SeriesInput {
  key: string;
  points: Float64Array;
  look: SeriesLook;
  visible: boolean;
}

export interface Grid {
  xPx: readonly number[];
  yPx: readonly number[];
  color: string;
}

export class PlotRenderer {
  private readonly app = new Application();
  private readonly grid = new Graphics();
  private readonly layer = new Container();
  private meshes = new Map<string, SeriesMesh>();
  private ready = false;
  private destroyed = false;
  private width = 1;
  private height = 1;
  /** Called after the GPU context is restored; Pixi re-uploads resources, we redraw. */
  onRestored: (() => void) | null = null;

  /** Creates the WebGL canvas inside `host`. Returns false if destroyed meanwhile. */
  async init(host: HTMLElement): Promise<boolean> {
    await this.app.init({
      width: this.width,
      height: this.height,
      antialias: true,
      backgroundAlpha: 0,
      autoDensity: true,
      resolution: window.devicePixelRatio || 1,
      preference: "webgl",
      autoStart: false,
      sharedTicker: false,
    });
    if (this.destroyed) {
      this.app.destroy(true);
      return false;
    }
    this.app.canvas.classList.add("plot-canvas");
    this.app.canvas.addEventListener("webglcontextrestored", () => {
      requestAnimationFrame(() => this.onRestored?.());
    });
    host.appendChild(this.app.canvas);
    this.app.stage.addChild(this.grid, this.layer);
    this.ready = true;
    return true;
  }

  resize(width: number, height: number) {
    this.width = Math.max(1, width);
    this.height = Math.max(1, height);
    if (this.ready) this.app.renderer.resize(this.width, this.height);
  }

  /** Replaces all series meshes. `origin` is the float32 reference point of the payload. */
  setSeries(series: readonly SeriesInput[], origin: Origin, style: "line" | "scatter") {
    this.clearSeries();
    for (const s of series) {
      const mesh = new SeriesMesh(s.points, origin, style, s.look);
      mesh.mesh.visible = s.visible && mesh.mesh.visible;
      this.meshes.set(s.key, mesh);
      this.layer.addChild(mesh.mesh);
    }
  }

  clearSeries() {
    for (const mesh of this.meshes.values()) mesh.destroy();
    this.meshes.clear();
    this.layer.removeChildren();
  }

  render(range: Range, grid: Grid) {
    if (!this.ready) return;
    this.grid.clear();
    for (const x of grid.xPx)
      this.grid.moveTo(Math.round(x) + 0.5, 0).lineTo(Math.round(x) + 0.5, this.height);
    for (const y of grid.yPx)
      this.grid.moveTo(0, Math.round(y) + 0.5).lineTo(this.width, Math.round(y) + 0.5);
    if (grid.xPx.length + grid.yPx.length > 0) this.grid.stroke({ width: 1, color: grid.color });
    for (const mesh of this.meshes.values()) mesh.setView(range, this.width, this.height);
    this.app.render();
  }

  destroy() {
    this.destroyed = true;
    if (!this.ready) return;
    this.clearSeries();
    this.app.destroy(true, { children: true });
    this.ready = false;
  }
}
