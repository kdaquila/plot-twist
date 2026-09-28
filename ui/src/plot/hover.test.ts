import { describe, expect, it } from "vitest";
import { HOVER_RADIUS_PX, nearestPoint } from "./hover";

// 100 × 100 px over data 0..100, so 1 data unit = 1 px (y grows upward).
const RANGE = { xMin: 0, xMax: 100, yMin: 0, yMax: 100 };
const pick = (series: { name: string; points: number[] }[], px: number, py: number) =>
  nearestPoint(
    series.map((s) => ({ name: s.name, points: Float64Array.from(s.points) })),
    RANGE,
    100,
    100,
    px,
    py,
  );

describe("hover readout (FR-009b, FR-009g)", () => {
  it("reports the closest point across series with its exact values", () => {
    const hit = pick(
      [
        { name: "a", points: [50, 50, 60, 60] },
        { name: "b", points: [52, 49] },
      ],
      52,
      51,
    );
    expect(hit).toMatchObject({ series: "b", x: 52, y: 49, px: 52, py: 51 });
  });

  it(`shows nothing beyond ${String(HOVER_RADIUS_PX)} px`, () => {
    expect(pick([{ name: "a", points: [50, 50] }], 50 + HOVER_RADIUS_PX + 1, 50)).toBeNull();
    expect(pick([{ name: "a", points: [50, 50] }], 50 + HOVER_RADIUS_PX - 1, 50)).not.toBeNull();
  });

  it("never reports missing values", () => {
    const hit = pick([{ name: "a", points: [50, NaN, 55, 50] }], 50, 50);
    expect(hit).toMatchObject({ x: 55, y: 50 });
  });

  it("considers only the series it is given (hidden series are left out)", () => {
    expect(pick([], 50, 50)).toBeNull();
  });
});
