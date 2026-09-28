// Decodes the binary view payload (contracts/tauri-commands.md).

export interface SeriesPayload {
  columnIndex: number;
  reduced: boolean;
  /** Min/max of the series within the requested X range; null if nothing is visible. */
  yExtent: [number, number] | null;
  /** Interleaved [x0, y0, x1, y1, …] in draw order; a NaN pair is a line break. */
  points: Float64Array;
}

export function decodeViewPayload(buffer: ArrayBuffer): SeriesPayload[] {
  const view = new DataView(buffer);
  let offset = 0;
  const u32 = () => {
    const value = view.getUint32(offset, true);
    offset += 4;
    return value;
  };
  const f64 = () => {
    const value = view.getFloat64(offset, true);
    offset += 8;
    return value;
  };
  const series: SeriesPayload[] = [];
  const count = u32();
  for (let s = 0; s < count; s++) {
    const columnIndex = u32();
    const reduced = view.getUint8(offset) === 1;
    offset += 1;
    const yMin = f64();
    const yMax = f64();
    const pointCount = u32();
    const points = new Float64Array(pointCount * 2);
    for (let i = 0; i < points.length; i++) points[i] = f64();
    series.push({
      columnIndex,
      reduced,
      yExtent: Number.isNaN(yMin) ? null : [yMin, yMax],
      points,
    });
  }
  return series;
}
