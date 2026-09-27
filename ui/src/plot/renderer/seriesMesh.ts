// Builds GPU meshes for one series: instanced segments (line) or instanced discs (scatter).
import { Buffer, BufferUsage, Geometry, Mesh, Shader, UniformGroup } from "pixi.js";
import type { AttributeOptions } from "pixi.js";
import { rgb } from "../legend/palette";
import type { Range } from "../viewState";
import { lineFragment, lineVertex, markerFragment, markerVertex } from "./shaders";

export const LINE_WIDTH_PX = 1.5;
export const MARKER_RADIUS_PX = 3;

const LINE_CORNERS = [0, -1, 1, -1, 0, 1, 1, 1];
const MARKER_CORNERS = [-1, -1, 1, -1, -1, 1, 1, 1];
const QUAD_INDICES = [0, 1, 2, 2, 1, 3];

export interface SeriesLook {
  color: string;
  alternate: boolean;
}

/** Data-space point used as the float32 origin for a payload. */
export interface Origin {
  x: number;
  y: number;
}

function uniforms(look: SeriesLook, style: "line" | "scatter") {
  const [r, g, b] = rgb(look.color);
  return new UniformGroup({
    uPlotScale: { value: new Float32Array([1, 1]), type: "vec2<f32>" },
    uPlotOffset: { value: new Float32Array([0, 0]), type: "vec2<f32>" },
    uPlotSize: { value: new Float32Array([1, 1]), type: "vec2<f32>" },
    uSeriesColor: { value: new Float32Array([r, g, b, 1]), type: "vec4<f32>" },
    uWidth: { value: LINE_WIDTH_PX, type: "f32" },
    uRadius: { value: MARKER_RADIUS_PX, type: "f32" },
    uDashed: { value: style === "line" && look.alternate ? 1 : 0, type: "f32" },
    uHollow: { value: style === "scatter" && look.alternate ? 1 : 0, type: "f32" },
  });
}

/** Segment endpoints `[x0, y0, x1, y1, …]` relative to `origin`, skipping breaks. */
function segments(points: Float64Array, origin: Origin): Float32Array {
  const out = new Float32Array(Math.max(0, points.length - 2) * 2);
  let n = 0;
  for (let i = 0; i + 3 < points.length; i += 2) {
    const x0 = points[i] ?? NaN;
    const y0 = points[i + 1] ?? NaN;
    const x1 = points[i + 2] ?? NaN;
    const y1 = points[i + 3] ?? NaN;
    if (Number.isNaN(x0) || Number.isNaN(y0) || Number.isNaN(x1) || Number.isNaN(y1)) continue;
    out[n++] = x0 - origin.x;
    out[n++] = y0 - origin.y;
    out[n++] = x1 - origin.x;
    out[n++] = y1 - origin.y;
  }
  return out.subarray(0, n);
}

function centers(points: Float64Array, origin: Origin): Float32Array {
  const out = new Float32Array(points.length);
  let n = 0;
  for (let i = 0; i + 1 < points.length; i += 2) {
    const x = points[i] ?? NaN;
    const y = points[i + 1] ?? NaN;
    if (Number.isNaN(x) || Number.isNaN(y)) continue;
    out[n++] = x - origin.x;
    out[n++] = y - origin.y;
  }
  return out.subarray(0, n);
}

export class SeriesMesh {
  readonly mesh: Mesh<Geometry, Shader>;
  private readonly group: UniformGroup;

  constructor(
    points: Float64Array,
    private readonly origin: Origin,
    style: "line" | "scatter",
    look: SeriesLook,
  ) {
    const isLine = style === "line";
    const data = isLine ? segments(points, origin) : centers(points, origin);
    const perInstance = isLine ? 4 : 2;
    const buffer = new Buffer({ data, usage: BufferUsage.VERTEX | BufferUsage.COPY_DST });
    const stride = perInstance * 4;
    const attribute = (offset: number) =>
      ({ buffer, format: "float32x2", stride, offset, instance: true }) as const;
    const attributes: AttributeOptions = isLine
      ? { aPosition: LINE_CORNERS, aStart: attribute(0), aEnd: attribute(8) }
      : { aPosition: MARKER_CORNERS, aCenter: attribute(0) };
    const geometry = new Geometry({
      attributes,
      indexBuffer: QUAD_INDICES,
      instanceCount: data.length / perInstance,
    });
    this.group = uniforms(look, style);
    const shader = Shader.from({
      gl: isLine
        ? { vertex: lineVertex, fragment: lineFragment, name: "plot-line" }
        : { vertex: markerVertex, fragment: markerFragment, name: "plot-marker" },
      resources: { plot: this.group },
    });
    this.mesh = new Mesh({ geometry, shader });
    this.mesh.visible = geometry.instanceCount > 0;
  }

  /** Updates the transform for the current view; cheap enough to call every frame. */
  setView(range: Range, width: number, height: number) {
    const u = this.group.uniforms as Record<string, Float32Array | number>;
    const sx = width / (range.xMax - range.xMin);
    const sy = -height / (range.yMax - range.yMin);
    const scale = u.uPlotScale as Float32Array;
    const offset = u.uPlotOffset as Float32Array;
    const resolution = u.uPlotSize as Float32Array;
    scale[0] = sx;
    scale[1] = sy;
    offset[0] = (this.origin.x - range.xMin) * sx;
    offset[1] = (this.origin.y - range.yMax) * sy;
    resolution[0] = width;
    resolution[1] = height;
  }

  destroy() {
    // Mesh.destroy() clears these references, so take them first.
    const { geometry, shader } = this.mesh;
    this.mesh.destroy({ children: true });
    geometry.destroy(true);
    shader?.destroy();
  }
}
