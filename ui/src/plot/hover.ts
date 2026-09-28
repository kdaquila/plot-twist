// Nearest-point search for the hover readout (FR-009b, FR-009g).
import type { Range } from "./viewState";

export const HOVER_RADIUS_PX = 8;

export interface HoverSeries {
  name: string;
  points: Float64Array;
}

export interface HoverHit {
  series: string;
  x: number;
  y: number;
  px: number;
  py: number;
}

/**
 * Finds the visible point closest to the pointer `(px, py)` (CSS pixels from the plot's
 * top-left) within 8 px across the given series. Missing values are never reported.
 */
export function nearestPoint(
  series: readonly HoverSeries[],
  range: Range,
  width: number,
  height: number,
  px: number,
  py: number,
): HoverHit | null {
  const sx = width / (range.xMax - range.xMin);
  const sy = height / (range.yMax - range.yMin);
  let best: HoverHit | null = null;
  let bestDistance = HOVER_RADIUS_PX * HOVER_RADIUS_PX;
  for (const s of series) {
    const p = s.points;
    for (let i = 0; i + 1 < p.length; i += 2) {
      const x = p[i] ?? NaN;
      const y = p[i + 1] ?? NaN;
      if (Number.isNaN(x) || Number.isNaN(y)) continue;
      const dx = (x - range.xMin) * sx - px;
      const dy = (range.yMax - y) * sy - py;
      const distance = dx * dx + dy * dy;
      if (distance <= bestDistance) {
        bestDistance = distance;
        best = { series: s.name, x, y, px: px + dx, py: py + dy };
      }
    }
  }
  return best;
}
