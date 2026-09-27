// Pure view-range math: fitting, zooming, panning, and zoom limits (FR-009, FR-009d–f).

export interface Range {
  xMin: number;
  xMax: number;
  yMin: number;
  yMax: number;
}

export interface Extent {
  min: number;
  max: number;
}

export const PADDING = 0.05;
export const WHEEL_FACTOR = 1.2;
const MIN_SPAN_RATIO = 1e-6;
const MAX_SPAN_RATIO = 100;

/** Union of extents, ignoring missing ones. */
export function union(extents: readonly (Extent | null)[]): Extent | null {
  let result: Extent | null = null;
  for (const e of extents) {
    if (!e) continue;
    result = result
      ? { min: Math.min(result.min, e.min), max: Math.max(result.max, e.max) }
      : { ...e };
  }
  return result;
}

/** Extent padded by 5% per side; a zero-width extent gets a sensible non-zero range. */
export function padded(extent: Extent | null): [number, number] {
  if (!extent) return [0, 1];
  const { min, max } = extent;
  if (max <= min) {
    const d = Math.max(Math.abs(min) * 0.1, 1);
    return [min - d, min + d];
  }
  const pad = (max - min) * PADDING;
  return [min - pad, max + pad];
}

export function fit(x: Extent | null, y: Extent | null): Range {
  const [xMin, xMax] = padded(x);
  const [yMin, yMax] = padded(y);
  return { xMin, xMax, yMin, yMax };
}

export interface Limits {
  minX: number;
  maxX: number;
  minY: number;
  maxY: number;
}

/** Zoom limits relative to the full (fitted) range. */
export function limitsFor(full: Range): Limits {
  const xSpan = full.xMax - full.xMin;
  const ySpan = full.yMax - full.yMin;
  return {
    minX: xSpan * MIN_SPAN_RATIO,
    maxX: xSpan * MAX_SPAN_RATIO,
    minY: ySpan * MIN_SPAN_RATIO,
    maxY: ySpan * MAX_SPAN_RATIO,
  };
}

function scaleAxis(
  min: number,
  max: number,
  anchor: number,
  factor: number,
  minSpan: number,
  maxSpan: number,
): [number, number] {
  const span = max - min;
  const next = Math.min(Math.max(span * factor, minSpan), maxSpan);
  const ratio = next / span;
  return [anchor - (anchor - min) * ratio, anchor + (max - anchor) * ratio];
}

/** Zooms both axes by `factor` (< 1 zooms in) keeping the data point `(ax, ay)` fixed. */
export function zoomAt(range: Range, ax: number, ay: number, factor: number, limits: Limits) {
  const [xMin, xMax] = scaleAxis(range.xMin, range.xMax, ax, factor, limits.minX, limits.maxX);
  const [yMin, yMax] = scaleAxis(range.yMin, range.yMax, ay, factor, limits.minY, limits.maxY);
  return { xMin, xMax, yMin, yMax };
}

export function pan(range: Range, dx: number, dy: number): Range {
  return {
    xMin: range.xMin + dx,
    xMax: range.xMax + dx,
    yMin: range.yMin + dy,
    yMax: range.yMax + dy,
  };
}

/** Zooms to a data-space rectangle, respecting the minimum spans. */
export function zoomToRect(rect: Range, limits: Limits): Range {
  const widen = (min: number, max: number, minSpan: number): [number, number] => {
    if (max - min >= minSpan) return [min, max];
    const c = (min + max) / 2;
    return [c - minSpan / 2, c + minSpan / 2];
  };
  const [xMin, xMax] = widen(rect.xMin, rect.xMax, limits.minX);
  const [yMin, yMax] = widen(rect.yMin, rect.yMax, limits.minY);
  return { xMin, xMax, yMin, yMax };
}
